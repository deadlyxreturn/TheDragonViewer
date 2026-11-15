//! Firestorm's catalogue of group abilities (indra/newview/skins/default/xui/
//! en/role_actions.xml, read by LLGroupMgr::parseRoleActions, originally
//! LGPL 2.1), generated: the sets in their order, each ability with its
//! power bit (roles_constants.h) and its texts (`tr!` keys).

/// One ability: its power bit, short and long texts.
pub struct Ability {
    pub bit: u32,
    pub name: &'static str,
    pub help: &'static str,
}

/// A set of abilities, its icon and its texts.
pub struct AbilitySet {
    pub icon: &'static str,
    pub name: &'static str,
    pub help: &'static str,
    pub abilities: Vec<Ability>,
}

/// The power bit of "Post links in chat" (GP_SESSION_POST_LINKS), shown
/// only where the region applies it.
pub const POST_LINKS_BIT: u32 = 52;

/// The catalogue; `post_links`: the region we are in applies the ability
/// to post links in group chat (SimulatorFeatures' GroupChatPostLinksEnabled).
/// Elsewhere the ability is hidden, as get_unsupported_powers hides it in
/// LLPanelGroupSubTab::buildActionCategory (indra/newview/llpanelgrouproles.cpp).
pub fn catalogue(post_links: bool) -> Vec<AbilitySet> {
    let mut sets = all();
    if !post_links {
        for set in &mut sets {
            set.abilities.retain(|a| a.bit != POST_LINKS_BIT);
        }
    }
    sets
}

fn all() -> Vec<AbilitySet> {
    vec![
        AbilitySet {
            icon: "users",
            name: prototype_i18n::tr!("group-abilities-membership"),
            help: prototype_i18n::tr!("group-abilities-membership-help"),
            abilities: vec![
                Ability {
                    bit: 1,
                    name: prototype_i18n::tr!("group-ability-member-invite"),
                    help: prototype_i18n::tr!("group-ability-member-invite-help"),
                },
                Ability {
                    bit: 2,
                    name: prototype_i18n::tr!("group-ability-member-eject"),
                    help: prototype_i18n::tr!("group-ability-member-eject-help"),
                },
                Ability {
                    bit: 51,
                    name: prototype_i18n::tr!("group-ability-allow-ban"),
                    help: prototype_i18n::tr!("group-ability-allow-ban-help"),
                },
                Ability {
                    bit: 3,
                    name: prototype_i18n::tr!("group-ability-member-options"),
                    help: prototype_i18n::tr!("group-ability-member-options-help"),
                },
            ],
        },
        AbilitySet {
            icon: "crown",
            name: prototype_i18n::tr!("group-abilities-roles"),
            help: prototype_i18n::tr!("group-abilities-roles-help"),
            abilities: vec![
                Ability {
                    bit: 4,
                    name: prototype_i18n::tr!("group-ability-role-create"),
                    help: prototype_i18n::tr!("group-ability-role-create-help"),
                },
                Ability {
                    bit: 5,
                    name: prototype_i18n::tr!("group-ability-role-delete"),
                    help: prototype_i18n::tr!("group-ability-role-delete-help"),
                },
                Ability {
                    bit: 6,
                    name: prototype_i18n::tr!("group-ability-role-properties"),
                    help: prototype_i18n::tr!("group-ability-role-properties-help"),
                },
                Ability {
                    bit: 7,
                    name: prototype_i18n::tr!("group-ability-role-assign-member-limited"),
                    help: prototype_i18n::tr!("group-ability-role-assign-member-limited-help"),
                },
                Ability {
                    bit: 8,
                    name: prototype_i18n::tr!("group-ability-role-assign-member"),
                    help: prototype_i18n::tr!("group-ability-role-assign-member-help"),
                },
                Ability {
                    bit: 9,
                    name: prototype_i18n::tr!("group-ability-role-remove-member"),
                    help: prototype_i18n::tr!("group-ability-role-remove-member-help"),
                },
                Ability {
                    bit: 10,
                    name: prototype_i18n::tr!("group-ability-role-change-actions"),
                    help: prototype_i18n::tr!("group-ability-role-change-actions-help"),
                },
            ],
        },
        AbilitySet {
            icon: "id-filled",
            name: prototype_i18n::tr!("group-abilities-group-identity"),
            help: prototype_i18n::tr!("group-abilities-group-identity-help"),
            abilities: vec![Ability {
                bit: 11,
                name: prototype_i18n::tr!("group-ability-group-change-identity"),
                help: prototype_i18n::tr!("group-ability-group-change-identity-help"),
            }],
        },
        AbilitySet {
            icon: "map-2",
            name: prototype_i18n::tr!("group-abilities-parcel-management"),
            help: prototype_i18n::tr!("group-abilities-parcel-management-help"),
            abilities: vec![
                Ability {
                    bit: 12,
                    name: prototype_i18n::tr!("group-ability-land-deed"),
                    help: prototype_i18n::tr!("group-ability-land-deed-help"),
                },
                Ability {
                    bit: 13,
                    name: prototype_i18n::tr!("group-ability-land-release"),
                    help: prototype_i18n::tr!("group-ability-land-release-help"),
                },
                Ability {
                    bit: 14,
                    name: prototype_i18n::tr!("group-ability-land-set-sale-info"),
                    help: prototype_i18n::tr!("group-ability-land-set-sale-info-help"),
                },
                Ability {
                    bit: 15,
                    name: prototype_i18n::tr!("group-ability-land-divide-join"),
                    help: prototype_i18n::tr!("group-ability-land-divide-join-help"),
                },
            ],
        },
        AbilitySet {
            icon: "map-pin-filled",
            name: prototype_i18n::tr!("group-abilities-parcel-identity"),
            help: prototype_i18n::tr!("group-abilities-parcel-identity-help"),
            abilities: vec![
                Ability {
                    bit: 17,
                    name: prototype_i18n::tr!("group-ability-land-find-places"),
                    help: prototype_i18n::tr!("group-ability-land-find-places-help"),
                },
                Ability {
                    bit: 18,
                    name: prototype_i18n::tr!("group-ability-land-change-identity"),
                    help: prototype_i18n::tr!("group-ability-land-change-identity-help"),
                },
                Ability {
                    bit: 19,
                    name: prototype_i18n::tr!("group-ability-land-set-landing-point"),
                    help: prototype_i18n::tr!("group-ability-land-set-landing-point-help"),
                },
            ],
        },
        AbilitySet {
            icon: "adjustments-filled",
            name: prototype_i18n::tr!("group-abilities-parcel-settings"),
            help: prototype_i18n::tr!("group-abilities-parcel-settings-help"),
            abilities: vec![
                Ability {
                    bit: 20,
                    name: prototype_i18n::tr!("group-ability-land-change-media"),
                    help: prototype_i18n::tr!("group-ability-land-change-media-help"),
                },
                Ability {
                    bit: 21,
                    name: prototype_i18n::tr!("group-ability-land-edit"),
                    help: prototype_i18n::tr!("group-ability-land-edit-help"),
                },
                Ability {
                    bit: 22,
                    name: prototype_i18n::tr!("group-ability-land-options"),
                    help: prototype_i18n::tr!("group-ability-land-options-help"),
                },
                Ability {
                    bit: 46,
                    name: prototype_i18n::tr!("group-ability-land-change-environment"),
                    help: prototype_i18n::tr!("group-ability-land-change-environment-help"),
                },
            ],
        },
        AbilitySet {
            icon: "bolt-filled",
            name: prototype_i18n::tr!("group-abilities-parcel-powers"),
            help: prototype_i18n::tr!("group-abilities-parcel-powers-help"),
            abilities: vec![
                Ability {
                    bit: 23,
                    name: prototype_i18n::tr!("group-ability-land-allow-edit-land"),
                    help: prototype_i18n::tr!("group-ability-land-allow-edit-land-help"),
                },
                Ability {
                    bit: 24,
                    name: prototype_i18n::tr!("group-ability-land-allow-fly"),
                    help: prototype_i18n::tr!("group-ability-land-allow-fly-help"),
                },
                Ability {
                    bit: 25,
                    name: prototype_i18n::tr!("group-ability-land-allow-create"),
                    help: prototype_i18n::tr!("group-ability-land-allow-create-help"),
                },
                Ability {
                    bit: 26,
                    name: prototype_i18n::tr!("group-ability-land-allow-direct-teleport"),
                    help: prototype_i18n::tr!("group-ability-land-allow-direct-teleport-help"),
                },
                Ability {
                    bit: 28,
                    name: prototype_i18n::tr!("group-ability-land-allow-set-home"),
                    help: prototype_i18n::tr!("group-ability-land-allow-set-home-help"),
                },
                Ability {
                    bit: 41,
                    name: prototype_i18n::tr!("group-ability-land-allow-host-event"),
                    help: prototype_i18n::tr!("group-ability-land-allow-host-event-help"),
                },
            ],
        },
        AbilitySet {
            icon: "lock-filled",
            name: prototype_i18n::tr!("group-abilities-parcel-access"),
            help: prototype_i18n::tr!("group-abilities-parcel-access-help"),
            abilities: vec![
                Ability {
                    bit: 29,
                    name: prototype_i18n::tr!("group-ability-land-manage-allowed"),
                    help: prototype_i18n::tr!("group-ability-land-manage-allowed-help"),
                },
                Ability {
                    bit: 30,
                    name: prototype_i18n::tr!("group-ability-land-manage-banned"),
                    help: prototype_i18n::tr!("group-ability-land-manage-banned-help"),
                },
                Ability {
                    bit: 31,
                    name: prototype_i18n::tr!("group-ability-land-manage-passes"),
                    help: prototype_i18n::tr!("group-ability-land-manage-passes-help"),
                },
                Ability {
                    bit: 32,
                    name: prototype_i18n::tr!("group-ability-land-admin"),
                    help: prototype_i18n::tr!("group-ability-land-admin-help"),
                },
            ],
        },
        AbilitySet {
            icon: "photo-filled",
            name: prototype_i18n::tr!("group-abilities-parcel-content"),
            help: prototype_i18n::tr!("group-abilities-parcel-content-help"),
            abilities: vec![
                Ability {
                    bit: 48,
                    name: prototype_i18n::tr!("group-ability-land-return-group-owned"),
                    help: prototype_i18n::tr!("group-ability-land-return-group-owned-help"),
                },
                Ability {
                    bit: 33,
                    name: prototype_i18n::tr!("group-ability-land-return-group-set"),
                    help: prototype_i18n::tr!("group-ability-land-return-group-set-help"),
                },
                Ability {
                    bit: 34,
                    name: prototype_i18n::tr!("group-ability-land-return-non-group"),
                    help: prototype_i18n::tr!("group-ability-land-return-non-group-help"),
                },
                Ability {
                    bit: 35,
                    name: prototype_i18n::tr!("group-ability-land-gardening"),
                    help: prototype_i18n::tr!("group-ability-land-gardening-help"),
                },
            ],
        },
        AbilitySet {
            icon: "package",
            name: prototype_i18n::tr!("group-abilities-object-management"),
            help: prototype_i18n::tr!("group-abilities-object-management-help"),
            abilities: vec![
                Ability {
                    bit: 36,
                    name: prototype_i18n::tr!("group-ability-object-deed"),
                    help: prototype_i18n::tr!("group-ability-object-deed-help"),
                },
                Ability {
                    bit: 38,
                    name: prototype_i18n::tr!("group-ability-object-manipulate"),
                    help: prototype_i18n::tr!("group-ability-object-manipulate-help"),
                },
                Ability {
                    bit: 39,
                    name: prototype_i18n::tr!("group-ability-object-set-sale"),
                    help: prototype_i18n::tr!("group-ability-object-set-sale-help"),
                },
            ],
        },
        AbilitySet {
            icon: "coin-filled",
            name: prototype_i18n::tr!("group-abilities-accounting"),
            help: prototype_i18n::tr!("group-abilities-accounting-help"),
            abilities: vec![Ability {
                bit: 40,
                name: prototype_i18n::tr!("group-ability-accounting-accountable"),
                help: prototype_i18n::tr!("group-ability-accounting-accountable-help"),
            }],
        },
        AbilitySet {
            icon: "speakerphone",
            name: prototype_i18n::tr!("group-abilities-notices"),
            help: prototype_i18n::tr!("group-abilities-notices-help"),
            abilities: vec![
                Ability {
                    bit: 42,
                    name: prototype_i18n::tr!("group-ability-notices-send"),
                    help: prototype_i18n::tr!("group-ability-notices-send-help"),
                },
                Ability {
                    bit: 43,
                    name: prototype_i18n::tr!("group-ability-notices-receive"),
                    help: prototype_i18n::tr!("group-ability-notices-receive-help"),
                },
            ],
        },
        AbilitySet {
            icon: "messages-filled",
            name: prototype_i18n::tr!("group-abilities-chat"),
            help: prototype_i18n::tr!("group-abilities-chat-help"),
            abilities: vec![
                Ability {
                    bit: 16,
                    name: prototype_i18n::tr!("group-ability-join-group-chat"),
                    help: prototype_i18n::tr!("group-ability-join-group-chat-help"),
                },
                Ability {
                    bit: 27,
                    name: prototype_i18n::tr!("group-ability-join-voice-chat"),
                    help: prototype_i18n::tr!("group-ability-join-voice-chat-help"),
                },
                Ability {
                    bit: 37,
                    name: prototype_i18n::tr!("group-ability-moderate-group-chat"),
                    help: prototype_i18n::tr!("group-ability-moderate-group-chat-help"),
                },
                Ability {
                    bit: POST_LINKS_BIT,
                    name: prototype_i18n::tr!("group-ability-chat-post-links"),
                    help: prototype_i18n::tr!("group-ability-chat-post-links-help"),
                },
            ],
        },
        AbilitySet {
            icon: "key-filled",
            name: prototype_i18n::tr!("group-abilities-experience-tools-experience"),
            help: prototype_i18n::tr!("group-abilities-experience-tools-experience-help"),
            abilities: vec![
                Ability {
                    bit: 49,
                    name: prototype_i18n::tr!("group-ability-experience-admin"),
                    help: prototype_i18n::tr!("group-ability-experience-admin-help"),
                },
                Ability {
                    bit: 50,
                    name: prototype_i18n::tr!("group-ability-experience-contributor"),
                    help: prototype_i18n::tr!("group-ability-experience-contributor-help"),
                },
            ],
        },
    ]
}
