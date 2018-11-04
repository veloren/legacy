use std::{
    thread,
    time::{Duration, SystemTime},
};

/*
 Clock helps keep a stable Ticks per Second over the time of a second
*/

pub struct TpsMeasure {
    smooth_period: Duration,
    last_tps_system_time: SystemTime,
    next_tps_system_time: SystemTime,
    last_tps: f64,
    tps_counter: u64,
}

pub struct Clock {
    system_time: SystemTime,
    debt_time: Duration,
    reference_duration: Duration,
}

impl TpsMeasure {
    pub fn new(smooth_period: Duration, clock: &Clock) -> TpsMeasure {
        TpsMeasure {
            smooth_period,
            last_tps_system_time: clock.system_time,
            next_tps_system_time: clock.system_time + smooth_period,
            last_tps: 1.0 / clock.reference_duration.as_float_secs(),
            tps_counter: 0,
        }
    }

    pub fn get_tps(&mut self, clock: &Clock) -> f64 {
        //calculate tps
        self.tps_counter += 1;
        if self.next_tps_system_time < clock.system_time {
            let tps_duration = clock.system_time.duration_since(self.last_tps_system_time).unwrap();
            // get real time elapsed since last tps, not only 5000 ms
            let seconds: f64 = tps_duration.as_float_secs();
            self.last_tps = self.tps_counter as f64 / seconds;
            self.tps_counter = 0;
            self.last_tps_system_time = clock.system_time;
            self.next_tps_system_time = self.last_tps_system_time + self.smooth_period;
            info!("tps: {}", self.last_tps);
        };
        self.last_tps
    }
}

impl Clock {
    pub fn new(reference_duration: Duration) -> Clock {
        Clock {
            system_time: SystemTime::now(),
            debt_time: Duration::from_nanos(0),
            reference_duration,
        }
    }

    // returns delta and timestamp
    pub fn delta(&self) -> (Duration, SystemTime) {
        let cur = SystemTime::now();
        let delta = cur.duration_since(self.system_time);
        (delta.unwrap(), cur)
    }

    pub fn tick(&mut self) {
        let delta = self.delta();
        if delta.0 < self.reference_duration {
            // sleep is only necessary if we are fast enough
            let sleep_time = self.reference_duration - delta.0;
            if self.debt_time > Duration::from_nanos(0) {
                if self.debt_time >= sleep_time {
                    self.debt_time -= sleep_time;
                } else {
                    //println!("dd {:?}  -  {:?} -  {:?}", self.reference_duration, sleep_time, self.debt_time);
                    let sleep_time = sleep_time - self.debt_time;
                    self.debt_time = Duration::from_nanos(0);
                    thread::sleep(sleep_time);
                }
            } else {
                //println!("ss {:?}  -  {:?}", self.reference_duration, sleep_time);
                thread::sleep(sleep_time);
            }
        } else {
            self.debt_time += delta.0 - self.reference_duration;
            warn!(
                "clock is running behind, current dept: {:?}, reference_duration: {:?}",
                self.debt_time, self.reference_duration
            );
        }
        self.system_time = SystemTime::now();
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.debt_time = Duration::from_nanos(0);
        self.system_time = SystemTime::now();
    }

    #[allow(dead_code)]
    pub fn reference_duration(&self) -> Duration { self.reference_duration }
}
