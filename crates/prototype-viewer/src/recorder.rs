//! Prototype's built-in video recorder. The renderer copies each frame
//! without waiting (`prototype_render::record`); a thread of this module
//! writes them at a steady frame rate to FFmpeg (shipped beside the viewer),
//! which encodes them on the graphics card when it can (NVENC, AMF, Quick
//! Sync, VideoToolbox), else with x264 on its fastest setting. The viewer's
//! own sound is written beside the video and joined to it at the end, so a
//! clip is one MP4 file in the chosen folder.

use prototype_render::record::{Frame, Mode, Request};
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// The recorder's settings (Preferences › Recording, and the Record window).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RecorderSettings {
    /// Frames per second of the video (30 or 60).
    pub fps: u32,
    /// 0 standard, 1 high, 2 best.
    pub quality: u8,
    /// What the video shows: 0 the 3D view alone, 1 with the bars (the
    /// windows left out), 2 the whole screen.
    pub content: u8,
    /// The mouse pointer is drawn into the video.
    pub cursor: bool,
    /// Motion blur in the video only: 0 off, 1 light, 2 medium, 3 strong.
    pub motion_blur: u8,
    /// The avatars' name tags are hidden while recording.
    pub hide_name_tags: bool,
    /// The viewer's sound is recorded with the video.
    pub sound: bool,
    /// The microphone is recorded with the video (your voice).
    pub microphone: bool,
    /// Where the videos go; empty: `<Videos>/Dragon Viewer`.
    pub folder: String,
}

impl Default for RecorderSettings {
    fn default() -> Self {
        RecorderSettings {
            fps: 60,
            quality: 1,
            content: 0,
            cursor: false,
            motion_blur: 0,
            hide_name_tags: false,
            sound: true,
            microphone: false,
            folder: String::new(),
        }
    }
}

impl RecorderSettings {
    pub fn folder(&self) -> PathBuf {
        if self.folder.trim().is_empty() {
            default_folder()
        } else {
            PathBuf::from(self.folder.trim())
        }
    }
}

/// `<Videos>/Dragon Viewer`, or the data folder when there is none.
pub fn default_folder() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|d| d.video_dir().map(Path::to_path_buf))
        .unwrap_or_else(|| crate::settings::config_dir().join("videos"))
        .join("Dragon Viewer")
}

/// What the recorder last did, for the toolbar and the Record window.
#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    Saved(PathBuf),
    Failed(String),
}

struct Session {
    started: Instant,
    mode: Mode,
    cursor: bool,
    sink: SyncSender<Frame>,
    worker: JoinHandle<Result<PathBuf, String>>,
}

#[derive(Default)]
pub struct Recorder {
    session: Option<Session>,
    /// The microphone to record (Preferences › Sound's input; empty: the
    /// default one).
    pub mic_device: String,
    /// A recording that is being finished (the sound joined) off the frame.
    finishing: Option<JoinHandle<Result<PathBuf, String>>>,
    pub notice: Option<(Notice, Instant)>,
}

impl Recorder {
    pub fn recording(&self) -> bool {
        self.session.is_some()
    }

    pub fn finishing(&self) -> bool {
        self.finishing.is_some()
    }

    /// Time recorded so far.
    pub fn elapsed(&self) -> Duration {
        self.session.as_ref().map_or(Duration::ZERO, |s| s.started.elapsed())
    }

    /// The request the renderer gets with every frame while recording:
    /// `windows` the windows' rectangles and `pointer` the mouse, in pixels.
    pub fn request(&self, windows: &[[u32; 4]], pointer: Option<[f32; 2]>) -> Option<Request> {
        self.session.as_ref().map(|s| Request {
            mode: s.mode,
            holes: if s.mode == Mode::Bars { windows.to_vec() } else { Vec::new() },
            cursor: if s.cursor { pointer } else { None },
            sink: s.sink.clone(),
        })
    }

    pub fn start(&mut self, settings: &RecorderSettings) {
        if self.session.is_some() || self.finishing.is_some() {
            return;
        }
        let Some(ffmpeg) = ffmpeg_path() else {
            self.notice = Some((Notice::Failed(prototype_i18n::tr!("recorder-no-ffmpeg").to_owned()), Instant::now()));
            return;
        };
        let folder = settings.folder();
        if let Err(e) = std::fs::create_dir_all(&folder) {
            self.notice = Some((Notice::Failed(format!("{}: {e}", folder.display())), Instant::now()));
            return;
        }
        // a few frames of slack: more and the video would lag behind
        let (sink, frames) = sync_channel::<Frame>(3);
        let audio = settings.sound.then(|| {
            let (tx, rx) = sync_channel::<Vec<f32>>(256);
            prototype_audio::set_record_tap(Some(tx));
            rx
        });
        let mic = settings.microphone.then(|| {
            let name = (!self.mic_device.trim().is_empty()).then(|| self.mic_device.clone());
            prototype_audio::record_microphone(name)
        });
        if matches!(mic, Some(None)) {
            log::warn!("recorder: no microphone could be opened");
        }
        let mic = mic.flatten();
        let job = Job {
            ffmpeg,
            folder,
            fps: settings.fps.clamp(15, 120),
            quality: settings.quality.min(2),
            blur: settings.motion_blur.min(3),
            name: format!("Prototype {}", chrono::Local::now().format("%Y-%m-%d %H-%M-%S")),
        };
        let worker = std::thread::Builder::new()
            .name("recorder".into())
            .spawn(move || run(job, frames, audio, mic))
            .map_err(|e| e.to_string());
        match worker {
            Ok(worker) => {
                log::info!("recorder: started ({} fps, quality {})", settings.fps, settings.quality);
                self.session = Some(Session {
                    started: Instant::now(),
                    mode: match settings.content {
                        0 => Mode::World,
                        1 => Mode::Bars,
                        _ => Mode::Everything,
                    },
                    cursor: settings.cursor,
                    sink,
                    worker,
                });
            }
            Err(e) => {
                prototype_audio::set_record_tap(None);
                self.notice = Some((Notice::Failed(e), Instant::now()));
            }
        }
    }

    /// Ends the recording: the thread writes what it has, joins the sound
    /// and reports (`poll`).
    pub fn stop(&mut self) {
        if let Some(s) = self.session.take() {
            prototype_audio::set_record_tap(None);
            // dropping the last sender ends the frames
            drop(s.sink);
            self.finishing = Some(s.worker);
        }
    }

    pub fn toggle(&mut self, settings: &RecorderSettings) {
        if self.recording() {
            self.stop();
        } else {
            self.start(settings);
        }
    }

    /// At exit: waits (at most `limit`) for the video being finished.
    pub fn wait(&mut self, limit: Duration) {
        let until = Instant::now() + limit;
        while self.finishing.as_ref().is_some_and(|w| !w.is_finished()) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(20));
        }
        self.poll();
    }

    /// Once a frame: a recording finished becomes its notice.
    pub fn poll(&mut self) {
        if self.finishing.as_ref().is_some_and(|w| w.is_finished())
            && let Some(w) = self.finishing.take()
        {
            let notice = match w.join() {
                Ok(Ok(path)) => {
                    log::info!("recorder: saved {}", path.display());
                    Notice::Saved(path)
                }
                Ok(Err(e)) => {
                    log::warn!("recorder: {e}");
                    Notice::Failed(e)
                }
                Err(_) => Notice::Failed("the recorder stopped".into()),
            };
            self.notice = Some((notice, Instant::now()));
        }
    }
}

/// FFmpeg beside the viewer's executable, else on the PATH.
pub fn ffmpeg_path() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        for candidate in [dir.join(exe), dir.join("tools").join(exe)] {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|d| d.join(exe)).find(|p| p.is_file())
}

fn command(ffmpeg: &Path) -> Command {
    let mut c = Command::new(ffmpeg);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // no console window flashing up
        c.creation_flags(0x0800_0000);
    }
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    c
}

/// The fastest H.264 encoder this PC has, with its settings for `quality`:
/// the graphics card's first, tried on a tiny clip.
fn pick_encoder(ffmpeg: &Path, quality: u8) -> Vec<String> {
    let q = [26, 21, 17][quality as usize];
    let candidates: Vec<(&str, Vec<String>)> = vec![
        ("h264_nvenc", vec!["-preset".into(), "p4".into(), "-tune".into(), "hq".into(), "-rc".into(), "vbr".into(), "-cq".into(), q.to_string(), "-b:v".into(), "0".into()]),
        ("h264_amf", vec!["-usage".into(), "transcoding".into(), "-quality".into(), "balanced".into(), "-rc".into(), "cqp".into(), "-qp_i".into(), q.to_string(), "-qp_p".into(), q.to_string()]),
        ("h264_qsv", vec!["-preset".into(), "faster".into(), "-global_quality".into(), q.to_string()]),
        ("h264_videotoolbox", vec!["-q:v".into(), (90 - q * 2).to_string()]),
        ("libx264", vec!["-preset".into(), "ultrafast".into(), "-tune".into(), "zerolatency".into(), "-crf".into(), q.to_string()]),
    ];
    for (name, args) in candidates {
        let ok = command(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "color=c=black:s=320x240:r=30", "-frames:v", "3", "-c:v", name])
            .args(&args)
            .args(["-pix_fmt", "yuv420p", "-f", "null", "-"])
            .status()
            .is_ok_and(|s| s.success());
        if ok {
            log::info!("recorder: encoding with {name}");
            let mut out = vec!["-c:v".to_owned(), name.to_owned()];
            out.extend(args);
            return out;
        }
    }
    vec!["-c:v".into(), "mpeg4".into(), "-q:v".into(), "3".into()]
}

struct Job {
    ffmpeg: PathBuf,
    folder: PathBuf,
    fps: u32,
    quality: u8,
    blur: u8,
    name: String,
}

/// The recorder's thread: frames in at whatever rate the viewer draws,
/// video out at a steady `fps` (a frame repeated or skipped to keep time).
fn run(
    job: Job,
    frames: Receiver<Frame>,
    audio: Option<Receiver<Vec<f32>>>,
    mic: Option<prototype_audio::MicRecording>,
) -> Result<PathBuf, String> {
    let encoder = pick_encoder(&job.ffmpeg, job.quality);
    let video = job.folder.join(format!("{}.video.mp4", job.name));
    let wav = job.folder.join(format!("{}.sound.wav", job.name));
    let out = job.folder.join(format!("{}.mp4", job.name));
    let mut sound = audio.map(|rx| (rx, WavWriter::create(&wav, 2, prototype_audio::ENGINE_SAMPLE_RATE)));
    let mic_wav = job.folder.join(format!("{}.mic.wav", job.name));
    let mut voice = mic.map(|m| {
        let w = WavWriter::create(&mic_wav, 1, m.rate);
        (m, w)
    });
    let mut child: Option<(Child, BufWriter<std::process::ChildStdin>, (u32, u32))> = None;
    let mut start = None;
    let mut written: u64 = 0;
    let mut failure = None;
    let frame_time = 1.0 / f64::from(job.fps);
    let mut blur = MotionBlur::new(job.blur);
    loop {
        // the sound that came meanwhile
        if let Some((rx, writer)) = &mut sound {
            while let Ok(samples) = rx.try_recv() {
                if let Ok(w) = writer {
                    let _ = w.write(&samples);
                }
            }
        }
        if let Some((m, writer)) = &mut voice {
            while let Ok(samples) = m.samples.try_recv() {
                if let Ok(w) = writer {
                    let _ = w.write(&samples);
                }
            }
        }
        let frame = match frames.recv_timeout(Duration::from_millis(50)) {
            Ok(f) => f,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };
        if child.is_none() {
            let (w, h) = (frame.width & !1, frame.height & !1);
            let spawned = command(&job.ffmpeg)
                .stdin(Stdio::piped())
                .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "rawvideo"])
                .args(["-pix_fmt", if frame.bgra { "bgra" } else { "rgba" }])
                .args(["-s", &format!("{}x{}", frame.width, frame.height)])
                .args(["-r", &job.fps.to_string(), "-i", "-"])
                .args(["-vf", &format!("crop={w}:{h}:0:0")])
                .args(&encoder)
                .args(["-pix_fmt", "yuv420p", "-movflags", "+faststart"])
                .arg(&video)
                .spawn();
            match spawned {
                Ok(mut c) => {
                    let Some(stdin) = c.stdin.take() else {
                        failure = Some("FFmpeg has no input".to_owned());
                        break;
                    };
                    child = Some((c, BufWriter::with_capacity(1 << 22, stdin), (frame.width, frame.height)));
                    start = Some(frame.at);
                }
                Err(e) => {
                    failure = Some(format!("FFmpeg: {e}"));
                    break;
                }
            }
        }
        let Some((_, pipe, size)) = &mut child else { break };
        // a window resized mid-recording: its frames do not fit the video
        if (frame.width, frame.height) != *size {
            continue;
        }
        let mut frame = frame;
        compose(&mut frame);
        blur.apply(&mut frame.pixels);
        let due = start.map_or(0, |s| (frame.at.saturating_duration_since(s).as_secs_f64() / frame_time) as u64 + 1);
        // at least this frame, repeated up to where the video should be
        let copies = due.saturating_sub(written).max(u64::from(written == 0));
        let mut broken = false;
        for _ in 0..copies.min(u64::from(job.fps)) {
            if pipe.write_all(&frame.pixels).is_err() {
                broken = true;
                break;
            }
            written += 1;
        }
        if broken {
            failure = Some("FFmpeg stopped".to_owned());
            break;
        }
    }
    // the rest of the sound
    let sound_ok = match sound {
        Some((rx, Ok(mut w))) => {
            while let Ok(samples) = rx.try_recv() {
                let _ = w.write(&samples);
            }
            let any = w.frames > 0;
            w.finish().is_ok() && any
        }
        _ => false,
    };
    let voice_ok = match voice {
        Some((m, Ok(mut w))) => {
            while let Ok(samples) = m.samples.try_recv() {
                let _ = w.write(&samples);
            }
            drop(m);
            let any = w.frames > 0;
            w.finish().is_ok() && any
        }
        _ => false,
    };
    let Some((mut c, pipe, _)) = child else {
        let _ = std::fs::remove_file(&wav);
        let _ = std::fs::remove_file(&mic_wav);
        return Err(failure.unwrap_or_else(|| "nothing was recorded".to_owned()));
    };
    drop(pipe);
    let status = c.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        let _ = std::fs::remove_file(&wav);
        let _ = std::fs::remove_file(&mic_wav);
        return Err(failure.unwrap_or_else(|| "FFmpeg could not encode the video".to_owned()));
    }
    // the sounds joined to the video (the viewer's and the microphone's
    // mixed), the video copied as it is (no second encoding)
    let inputs: Vec<&PathBuf> = [(sound_ok, &wav), (voice_ok, &mic_wav)].into_iter().filter(|(ok, _)| *ok).map(|(_, p)| p).collect();
    if !inputs.is_empty() {
        let mut c = command(&job.ffmpeg);
        c.args(["-hide_banner", "-loglevel", "error", "-y", "-i"]).arg(&video);
        for i in &inputs {
            c.arg("-i").arg(i);
        }
        if inputs.len() == 2 {
            c.args(["-filter_complex", "[1:a][2:a]amix=inputs=2:duration=longest:normalize=0[a]", "-map", "0:v", "-map", "[a]"]);
        }
        c.args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-shortest", "-movflags", "+faststart"]).arg(&out);
        if c.status().is_ok_and(|s| s.success()) {
            let _ = std::fs::remove_file(&video);
            let _ = std::fs::remove_file(&wav);
            let _ = std::fs::remove_file(&mic_wav);
            return Ok(out);
        }
    }
    let _ = std::fs::remove_file(&wav);
    let _ = std::fs::remove_file(&mic_wav);
    std::fs::rename(&video, &out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// The windows' holes filled with the 3D view behind them, then the mouse
/// pointer drawn over.
fn compose(frame: &mut Frame) {
    let (w, h) = (frame.width as usize, frame.height as usize);
    if let Some(scene) = &frame.scene {
        for &[x0, y0, x1, y1] in &frame.holes {
            let (x0, x1) = ((x0 as usize).min(w), (x1 as usize).min(w));
            let (y0, y1) = ((y0 as usize).min(h), (y1 as usize).min(h));
            if x1 <= x0 {
                continue;
            }
            for y in y0..y1 {
                let a = (y * w + x0) * 4;
                let b = (y * w + x1) * 4;
                frame.pixels[a..b].copy_from_slice(&scene[a..b]);
            }
        }
    }
    if let Some([cx, cy]) = frame.cursor {
        draw_cursor(&mut frame.pixels, w, h, cx.round() as i32, cy.round() as i32);
    }
}

/// Motion blur of the video: each frame keeps part of the frames before
/// it (an exponential trail), so what moves streaks and what stands still
/// stays sharp. Done here, on the recorder's thread: the screen and the
/// frame rate are not touched.
struct MotionBlur {
    /// How much of the trail stays, out of 256.
    keep: u16,
    trail: Vec<u8>,
}

impl MotionBlur {
    fn new(level: u8) -> Self {
        let keep = match level {
            1 => 90,
            2 => 140,
            3 => 185,
            _ => 0,
        };
        MotionBlur { keep, trail: Vec::new() }
    }

    fn apply(&mut self, pixels: &mut [u8]) {
        if self.keep == 0 {
            return;
        }
        if self.trail.len() != pixels.len() {
            self.trail = pixels.to_vec();
            return;
        }
        let keep = self.keep;
        let take = 256 - keep;
        for (t, p) in self.trail.iter_mut().zip(pixels.iter_mut()) {
            let v = ((u16::from(*t) * keep + u16::from(*p) * take) >> 8) as u8;
            *t = v;
            *p = v;
        }
    }
}

/// The usual arrow pointer: `#` its black edge, `.` its white inside.
const ARROW: [&str; 19] = [
    "#",
    "##",
    "#.#",
    "#..#",
    "#...#",
    "#....#",
    "#.....#",
    "#......#",
    "#.......#",
    "#........#",
    "#.........#",
    "#......#####",
    "#...#..#",
    "#..##..#",
    "#.#  #..#",
    "##   #..#",
    "#     #..#",
    "      #..#",
    "       ##",
];

fn draw_cursor(pixels: &mut [u8], w: usize, h: usize, x: i32, y: i32) {
    // larger on a large screen, as the system draws it
    let scale = if h >= 1400 { 2 } else { 1 };
    for (r, line) in ARROW.iter().enumerate() {
        for (c, ch) in line.chars().enumerate() {
            let v = match ch {
                '#' => 0u8,
                '.' => 255u8,
                _ => continue,
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x + (c * scale + dx) as i32;
                    let py = y + (r * scale + dy) as i32;
                    if px < 0 || py < 0 || px as usize >= w || py as usize >= h {
                        continue;
                    }
                    let i = (py as usize * w + px as usize) * 4;
                    pixels[i] = v;
                    pixels[i + 1] = v;
                    pixels[i + 2] = v;
                    pixels[i + 3] = 255;
                }
            }
        }
    }
}

/// A sound as a 16-bit WAV (the viewer's in stereo at the engine's rate,
/// the microphone's in mono at its own).
struct WavWriter {
    file: BufWriter<std::fs::File>,
    frames: u64,
    channels: u16,
    rate: u32,
}

impl WavWriter {
    fn create(path: &Path, channels: u16, rate: u32) -> std::io::Result<Self> {
        let mut file = BufWriter::new(std::fs::File::create(path)?);
        file.write_all(&[0u8; 44])?;
        Ok(WavWriter { file, frames: 0, channels: channels.max(1), rate: rate.max(1) })
    }

    fn write(&mut self, samples: &[f32]) -> std::io::Result<()> {
        for s in samples {
            let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            self.file.write_all(&v.to_le_bytes())?;
        }
        self.frames += (samples.len() / usize::from(self.channels)) as u64;
        Ok(())
    }

    fn finish(mut self) -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom};
        let block = u32::from(self.channels) * 2;
        let data = (self.frames * u64::from(block)) as u32;
        let mut h = Vec::with_capacity(44);
        h.extend_from_slice(b"RIFF");
        h.extend_from_slice(&(36 + data).to_le_bytes());
        h.extend_from_slice(b"WAVEfmt ");
        h.extend_from_slice(&16u32.to_le_bytes());
        h.extend_from_slice(&1u16.to_le_bytes());
        h.extend_from_slice(&self.channels.to_le_bytes());
        h.extend_from_slice(&self.rate.to_le_bytes());
        h.extend_from_slice(&(self.rate * block).to_le_bytes());
        h.extend_from_slice(&(block as u16).to_le_bytes());
        h.extend_from_slice(&16u16.to_le_bytes());
        h.extend_from_slice(b"data");
        h.extend_from_slice(&data.to_le_bytes());
        self.file.flush()?;
        let mut f = self.file.into_inner().map_err(|e| e.into_error())?;
        f.seek(SeekFrom::Start(0))?;
        f.write_all(&h)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_blur_keeps_a_trail_and_still_pixels_stay() {
        let mut b = MotionBlur::new(2);
        let mut first = vec![200u8; 8];
        b.apply(&mut first);
        assert_eq!(first, vec![200u8; 8]);
        let mut moved = vec![0u8; 8];
        b.apply(&mut moved);
        assert!(moved.iter().all(|v| *v > 90 && *v < 120));
        let mut off = vec![7u8; 4];
        MotionBlur::new(0).apply(&mut off);
        assert_eq!(off, vec![7u8; 4]);
    }

    #[test]
    fn a_wav_header_counts_its_frames() {
        let dir = std::env::temp_dir().join(format!("proto-rec-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.wav");
        let mut w = WavWriter::create(&path, 2, 48_000).unwrap();
        w.write(&[0.0, 0.5, -0.5, 1.0]).unwrap();
        w.finish().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 8);
        assert_eq!(bytes.len(), 44 + 8);
        let _ = std::fs::remove_dir_all(dir);
    }
}
