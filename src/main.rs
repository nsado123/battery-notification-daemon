use battery::{Manager, State};
use notify_rust::Notification;
use notify_rust::Hint;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::time::{sleep, Duration};

const LOW_BATTERY_THRESHOLD: f32 = 25.0;
const CRITICAL_BATTERY_THRESHOLD: f32 = 10.0;
const FULL_BATTERY_THRESHOLD: f32 = 100.0;

// Define SVG icon paths
const CRITICAL_ICON: &str = "/usr/share/icons/critical.svg";
const LOW_ICON: &str = "/usr/share/icons/low-battery.svg";
const FULL_ICON: &str = "/usr/share/icons/full-battery.svg";
const CHARGING_ICON: &str = "/usr/share/icons/charging.svg";
const DISCHARGING_ICON: &str = "/usr/share/icons/unplugged.svg";

#[tokio::main]
async fn main() -> battery::Result<()> {
    let manager = Manager::new()?;
    let should_exit = Arc::new(AtomicBool::new(false));
    let exit_flag = should_exit.clone();

    // Spawn a task to handle Ctrl+C
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Received Ctrl+C, exiting gracefully...");
        exit_flag.store(true, Ordering::Relaxed);
    });

    let mut last_status = String::new();
    let mut last_capacity_level = String::new();

    while !should_exit.load(Ordering::Relaxed) {
        if let Some(Ok(battery)) = manager.batteries()?.next() {
            let capacity_percent = battery.state_of_charge().value * 100.0;
            let status_text = match battery.state() {
                State::Charging => "Charging",
                State::Discharging => "Discharging",
                State::Full => "Full",
                _ => "Unknown",
            };

            if status_text != last_status {
                match status_text {
                    "Charging" => {
                        send_notification(
                            "Charger Connected",
                            &format!("Charging ({:.0}%).", capacity_percent),
                            CHARGING_ICON,
                        );
                    }
                    "Discharging" => {
                        send_notification(
                            "Charger Disconnected",
                            &format!("Battery ({:.0}%).", capacity_percent),
                            DISCHARGING_ICON,
                        );
                    }
                    "Full" => {
                        send_notification(
                            "Battery Full",
                            "Battery is fully charged.",
                            FULL_ICON,
                        );
                    }
                    _ => {}
                }
                last_status = status_text.to_string();
            }

            let current_level = if capacity_percent >= FULL_BATTERY_THRESHOLD {
                "full"
            } else if capacity_percent <= CRITICAL_BATTERY_THRESHOLD {
                "critical"
            } else if capacity_percent <= LOW_BATTERY_THRESHOLD {
                "low"
            } else {
                "normal"
            };

            if current_level != last_capacity_level {
                match current_level {
                    "critical" => {
                        send_notification(
                            "Critical Battery",
                            &format!(
                                "Battery level is at {:.0}%. Plug in immediately!",
                                capacity_percent
                            ),
                            CRITICAL_ICON,
                        );
                    }
                    "low" => {
                        send_notification(
                            "Low Battery",
                            &format!(
                                "Battery level is at {:.0}%. Consider plugging in soon.",
                                capacity_percent
                            ),
                            LOW_ICON,
                        );
                    }
                    "full" => {
                        if status_text != "Full" {
                            send_notification(
                                "Battery Full",
                                &format!("Battery is at {:.0}%.", capacity_percent),
                                FULL_ICON,
                            );
                        }
                    }
                    _ => {}
                }
                last_capacity_level = current_level.to_string();
            }
        }

        sleep(Duration::from_secs(1)).await;
    }

    println!("Exiting program. Goodbye!");
    Ok(())
}

fn send_notification(summary: &str, body: &str, icon_path: &str) {
    let _ = Notification::new()
        .summary(summary)
        .body(body)
        .icon(icon_path) // Include the SVG icon
        .timeout(5000) // milliseconds
        .hint(Hint::Transient(true)) // Mark the notification as transient
        .show();
}
