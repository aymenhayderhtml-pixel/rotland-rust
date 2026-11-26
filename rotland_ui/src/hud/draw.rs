use engine::egui::{self, pos2, Align2, Color32, FontId};

use super::{
    ammo, clock, compass, crosshair, damage, death, downed, events, fps, hotbar, key_bar,
    notification, pickup, region_banner, safe_zone, scope, text, vehicle, vitals, warmth, HudConfig, HudState, IconCache,
};

/// Draws the HUD overlay using egui painter with explicit IconCache.
pub fn draw_hud(ctx: &egui::Context, s: &HudState, icons: &mut IconCache) {
    egui::Area::new(egui::Id::unique("rotland_hud"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let screen = ctx.content_rect();
            let scale = game::hud_math::hud_scale(screen.height());
            let time = ctx.input(|i| i.time);
            let config = HudConfig::load_default();

            // 1. Rifle scope overlay when aiming scoped weapon (underlay)
            if s.scoped && !s.dead {
                scope::draw_scope_overlay(painter, &screen);
            }

            // 2. FPS counter (below reserved 180px top-left minimap box, #7)
            fps::draw_fps(painter, &screen, s.fps, scale);

            // 3. Clock "Day N · HH:MM" top right
            clock::draw_clock(painter, &screen, s.hour, s.day, scale);

            // 4. Compass strip top centre (with camp indicator and waypoint diamond)
            compass::draw_compass(
                painter,
                &screen,
                s.heading_yaw,
                s.in_safe_zone,
                s.camp.as_ref(),
                s.waypoint.as_ref(),
                scale,
            );

            // 5. "Safe zone" label below compass when inside settlement
            safe_zone::draw_safe_zone(painter, &screen, s.in_safe_zone, scale);
            region_banner::draw_region_banner(painter, &screen, s.region_banner.as_ref(), scale);

            // 6. Crosshair in screen center (hidden when holding melee, scoped, driving, downed, or dead)
            if s.vehicle.is_none()
                && s.downed.is_none()
                && game::hud_math::is_crosshair_visible(s.dead, s.scoped, s.held)
            {
                crosshair::draw_crosshair(painter, screen.center(), scale);
            }

            // 7. Vital bars (Health & Stamina) bottom left (polished with rounded ends, darker back, recent damage trail, low pulse)
            vitals::draw_vital_bars_polished(
                painter,
                &screen,
                s.health,
                s.max_health,
                s.stamina,
                s.max_stamina,
                s.vitals_trail.as_ref(),
                time,
                scale,
                &config,
            );

            // 8. Need bars (Food & Water) above health (polished with rounded ends, darker back, recent damage trail, low pulse)
            vitals::draw_need_bars_polished(
                painter,
                &screen,
                time,
                s.food,
                s.water,
                s.vitals_trail.as_ref(),
                scale,
                &config,
            );

            // 8b. Body warmth bar (R32-05)
            warmth::draw_warmth_bar(painter, &screen, time, s.warmth.as_ref(), scale, &config);

            // 9. Pet Dog HUD strip above needs
            vitals::draw_dog_hud(painter, &screen, time, s.dog.as_ref(), scale);

            // 10. Hotbar bottom centre with item icon pictures, counts, and weapon condition bars
            hotbar::draw_hotbar(
                painter,
                ctx,
                &screen,
                &s.hotbar,
                s.hotbar_item_ids.as_deref(),
                s.hotbar_conditions.as_deref(),
                icons,
                scale,
            );

            // 11. Ammo readout bottom right (with dynamic key prompt), or the driving dashboard
            if let Some(info) = &s.vehicle {
                if !s.dead {
                    vehicle::draw_vehicle_hud(painter, &screen, time, info, scale);
                }
            } else {
                ammo::draw_ammo(painter, &screen, s.ammo, &s.key_labels.attack, scale);
            }

            // 12. Key help strip bottom right above ammo
            key_bar::draw_key_bar(painter, &screen, s.key_labels.key_help.as_deref(), scale);

            // 13. Centered toast/message if active
            if let Some(msg) = &s.message
                && !s.dead
                && s.downed.is_none()
            {
                text::draw_outline_text(
                    painter,
                    pos2(screen.center().x, screen.center().y + 50.0 * scale),
                    Align2::CENTER_CENTER,
                    msg,
                    FontId::proportional(18.0 * scale),
                    Color32::WHITE,
                );
            }

            // 14. Event notifications queue top-center under compass
            if let Some(notifs) = &s.notifications
                && !notifs.is_empty()
                && !s.dead
            {
                notification::draw_notifications(
                    painter,
                    &screen,
                    notifs,
                    time,
                    scale,
                    &config,
                );
            }

            // 15. Pickup popups stacked above hotbar on the right
            if let Some(popups) = &s.pickup_popups
                && !popups.is_empty()
                && !s.dead
            {
                pickup::draw_pickup_popups(
                    painter,
                    ctx,
                    &screen,
                    popups,
                    icons,
                    time,
                    scale,
                    &config,
                );
            }

            // 16. Damage direction indicators (red arcs on screen edge pointing to attackers)
            if let Some(indicators) = &s.damage_indicators
                && !indicators.is_empty()
                && !s.dead
            {
                let ppos = s.player_pos.unwrap_or(glam::Vec3::ZERO);
                damage::draw_damage_indicators(
                    painter,
                    &screen,
                    indicators,
                    ppos,
                    s.heading_yaw,
                    time,
                    scale,
                    &config,
                );
            }

            // 17. Horde near direction arrows
            if let Some(horde_arrows) = &s.horde_indicators
                && !horde_arrows.is_empty()
                && !s.dead
            {
                let ppos = s.player_pos.unwrap_or(glam::Vec3::ZERO);
                events::draw_horde_arrows(
                    painter,
                    &screen,
                    horde_arrows,
                    ppos,
                    s.heading_yaw,
                    time,
                    scale,
                    &config,
                );
            }

            // 18. Downed bar when knocked down, or death overlay when dead
            if s.dead {
                death::draw_death_overlay(painter, &screen, s.message.as_deref(), scale);
            } else if let Some(info) = &s.downed {
                downed::draw_downed_bar(painter, &screen, info, scale);
            }
        });
}
