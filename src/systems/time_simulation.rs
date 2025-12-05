use bevy::prelude::*;

/// Seasons with short in-game durations (RimWorld-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub const ORDER: [Season; 4] = [Season::Spring, Season::Summer, Season::Autumn, Season::Winter];

    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

/// Tracks the simulation calendar.
#[derive(Resource, Debug)]
pub struct Calendar {
    pub day: u32,
    pub day_of_season: u32,
    pub season: Season,
    pub time_of_day_hours: f32, // 0.0-24.0
    pub day_length_seconds: f32,
    pub season_length_days: u32,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            day: 1,
            day_of_season: 1,
            season: Season::Spring,
            time_of_day_hours: 8.0, // Morning start
            day_length_seconds: 300.0, // 5 real minutes per day
            season_length_days: 5,     // Short seasons for simulation pacing
        }
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct DayPassed {
    pub day: u32,
    pub season: Season,
    pub day_of_season: u32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SeasonChanged {
    pub season: Season,
}

pub struct TimeSimulationPlugin;

impl Plugin for TimeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Calendar>()
            .add_event::<DayPassed>()
            .add_event::<SeasonChanged>()
            .add_systems(Update, advance_calendar);
    }
}

fn advance_calendar(
    time: Res<Time<Virtual>>,
    mut calendar: ResMut<Calendar>,
    mut day_events: EventWriter<DayPassed>,
    mut season_events: EventWriter<SeasonChanged>,
) {
    let delta_hours = (time.delta_secs() / calendar.day_length_seconds) * 24.0;
    if delta_hours == 0.0 {
        return;
    }

    calendar.time_of_day_hours += delta_hours;

    while calendar.time_of_day_hours >= 24.0 {
        calendar.time_of_day_hours -= 24.0;
        calendar.day += 1;
        calendar.day_of_season += 1;

        // Season rollover
        if calendar.day_of_season > calendar.season_length_days {
            calendar.day_of_season = 1;
            calendar.season = calendar.season.next();
            season_events.send(SeasonChanged {
                season: calendar.season,
            });
        }

        day_events.send(DayPassed {
            day: calendar.day,
            season: calendar.season,
            day_of_season: calendar.day_of_season,
        });
    }
}
