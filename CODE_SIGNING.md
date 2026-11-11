# Dragon Viewer Windows packages

Dragon Viewer's Windows build currently produces unsigned packages. The
project publishes SHA-256 digests alongside the installer and portable ZIP.
Compare the package digest with the matching release asset using
`Get-FileHash -Algorithm SHA256`. The Windows workflow and build script
are available in this repository.
