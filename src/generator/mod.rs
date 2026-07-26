use rand::Rng;

use crate::model::{TargetSystem, TenantProfile, TrafficEvent, TrafficPattern};

#[derive(Debug, Clone)]
struct EventDraft {
    timestamp_ms: u64,
    tenant_id: String,
    payload_size_bytes: u32,
}

pub fn generate(profiles: &[TenantProfile], target_system: TargetSystem) -> Vec<TrafficEvent> {
    let mut drafts: Vec<EventDraft> = Vec::new();

    for profile in profiles {
        let mut tenant_events = match profile.traffic_pattern {
            TrafficPattern::Steady => generate_steady(profile),
            TrafficPattern::Burst => generate_burst(profile),
            TrafficPattern::Random => generate_random(profile),
        };
        drafts.append(&mut tenant_events);
    }

    drafts.sort_by(|a, b| {
        a.timestamp_ms
            .cmp(&b.timestamp_ms)
            .then_with(|| a.tenant_id.cmp(&b.tenant_id))
    });

    drafts
        .into_iter()
        .enumerate()
        .map(|(idx, draft)| {
            TrafficEvent::new(
                draft.timestamp_ms,
                &draft.tenant_id,
                (idx as u64) + 1,
                draft.payload_size_bytes,
                target_system.as_str(),
            )
        })
        .collect()
}

fn generate_steady(profile: &TenantProfile) -> Vec<EventDraft> {
    let interval_ms = (1000_u64 / u64::from(profile.requests_per_second.max(1))).max(1);
    let duration_ms = u64::from(profile.duration_seconds).saturating_mul(1000);

    let mut events = Vec::new();
    let mut timestamp_ms = 0_u64;

    while timestamp_ms < duration_ms {
        events.push(EventDraft {
            timestamp_ms,
            tenant_id: profile.tenant_id.clone(),
            payload_size_bytes: profile.payload_size_bytes,
        });
        timestamp_ms = timestamp_ms.saturating_add(interval_ms);
    }

    events
}

fn generate_burst(profile: &TenantProfile) -> Vec<EventDraft> {
    let burst_size = profile.burst_size.max(1);
    let burst_interval_ms = profile.burst_interval_ms.max(1);
    let duration_ms = u64::from(profile.duration_seconds).saturating_mul(1000);

    let mut events = Vec::new();
    let mut burst_start_ms = 0_u64;

    while burst_start_ms < duration_ms {
        for _ in 0..burst_size {
            events.push(EventDraft {
                timestamp_ms: burst_start_ms,
                tenant_id: profile.tenant_id.clone(),
                payload_size_bytes: profile.payload_size_bytes,
            });
        }

        burst_start_ms = burst_start_ms.saturating_add(burst_interval_ms);
    }

    events
}

fn generate_random(profile: &TenantProfile) -> Vec<EventDraft> {
    let duration_ms = u64::from(profile.duration_seconds).saturating_mul(1000);
    let min_interval = profile.random_min_interval_ms.max(1);
    let max_interval = profile.random_max_interval_ms.max(min_interval);

    let mut events = Vec::new();
    let mut timestamp_ms = 0_u64;
    let mut rng = rand::thread_rng();

    while timestamp_ms < duration_ms {
        events.push(EventDraft {
            timestamp_ms,
            tenant_id: profile.tenant_id.clone(),
            payload_size_bytes: profile.payload_size_bytes,
        });

        let step = if min_interval == max_interval {
            min_interval
        } else {
            rng.gen_range(min_interval..=max_interval)
        };

        timestamp_ms = timestamp_ms.saturating_add(step);
    }

    events
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::generate;
    use crate::model::{TargetSystem, TenantProfile, TrafficPattern};

    #[test]
    fn steady_events_are_evenly_spaced() {
        let profile = TenantProfile::new(
            "steady_a",
            "Steady A",
            TrafficPattern::Steady,
            10,
            128,
            1,
            1,
        );
        let events = generate(&[profile], TargetSystem::Generic);

        assert!(events.len() > 1);

        let intervals: Vec<u64> = events
            .windows(2)
            .map(|pair| pair[1].timestamp_ms - pair[0].timestamp_ms)
            .collect();

        let first = intervals[0];
        assert!(intervals.iter().all(|interval| *interval == first));
    }

    #[test]
    fn burst_events_follow_spike_and_quiet_pattern() {
        let profile =
            TenantProfile::new("burst_a", "Burst A", TrafficPattern::Burst, 20, 128, 1, 3)
                .with_burst_config(4, 500);
        let events = generate(&[profile], TargetSystem::Generic);

        let mut grouped: HashMap<u64, usize> = HashMap::new();
        for event in events {
            *grouped.entry(event.timestamp_ms).or_insert(0) += 1;
        }

        let mut timestamps: Vec<u64> = grouped.keys().copied().collect();
        timestamps.sort_unstable();

        assert!(timestamps.len() >= 2);
        for ts in &timestamps {
            assert_eq!(grouped[ts], 4);
        }

        let gaps: Vec<u64> = timestamps
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect();
        assert!(gaps.iter().all(|gap| *gap == 500));
    }

    #[test]
    fn random_events_stay_within_interval_bounds() {
        let min_interval = 40;
        let max_interval = 90;
        let profile = TenantProfile::new(
            "random_a",
            "Random A",
            TrafficPattern::Random,
            10,
            128,
            1,
            2,
        )
        .with_random_interval_config(min_interval, max_interval);
        let events = generate(&[profile], TargetSystem::Generic);

        assert!(events.len() > 1);

        for pair in events.windows(2) {
            let delta = pair[1].timestamp_ms - pair[0].timestamp_ms;
            assert!(delta >= min_interval && delta <= max_interval);
        }
    }

    #[test]
    fn request_ids_are_globally_monotonic() {
        let profiles = vec![
            TenantProfile::new("t1", "A", TrafficPattern::Steady, 5, 128, 1, 1),
            TenantProfile::new("t2", "B", TrafficPattern::Burst, 5, 128, 1, 1)
                .with_burst_config(2, 250),
        ];

        let events = generate(&profiles, TargetSystem::Generic);
        assert!(!events.is_empty());

        for (idx, event) in events.iter().enumerate() {
            assert_eq!(event.request_id, (idx as u64) + 1);
        }
    }

    #[test]
    fn generated_events_include_selected_target_system() {
        let profile = TenantProfile::new(
            "nginx_a",
            "NGINX A",
            TrafficPattern::Steady,
            5,
            128,
            1,
            1,
        );

        let events = generate(&[profile], TargetSystem::Nginx);
        assert!(!events.is_empty());
        assert!(events.iter().all(|event| event.target_system == "NGINX"));
    }
}
