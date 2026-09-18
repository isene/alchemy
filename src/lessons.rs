//! The experiments. Each one names a thing to make happen and says
//! enough to get you started. The check looks at the bench, so there is
//! only one way to pass: do it.

use crate::chem::Vessel;

pub struct Lesson {
    pub title: &'static str,
    pub goal: &'static str,
    pub hint: &'static str,
    /// True once a vessel on the bench shows what the goal asked for.
    pub done: fn(&Vessel) -> bool,
}

pub static LESSONS: &[Lesson] = &[
    Lesson {
        title: "The blue",
        goal: "Make a blue solution.",
        hint: "Water, then copper sulfate. This is the blue in every chemistry set, \
               and the first thing most chemists ever made.",
        done: |v| v.amount("CuSO4") > 1.0 && v.volume() > 1.0,
    },
    Lesson {
        title: "Iodine on starch",
        goal: "Turn a glass of starch water blue black.",
        hint: "Water, starch, then a flake of iodine. The iodine slips inside the \
               starch and the colour is so strong that a trace shows up.",
        done: |v| v.blue() > 0.5,
    },
    Lesson {
        title: "Make table salt",
        goal: "Make salt from an acid and a base, then boil the water off to see it.",
        hint: "Hydrochloric acid and caustic soda in the same glass, in equal measures. \
               Both are nasty; what they leave is on your dinner. Then light the burner \
               and wait for the water to go.",
        done: |v| v.amount("NaCl") > 5.0 && v.volume() < 0.5,
    },
    Lesson {
        title: "Find the chloride",
        goal: "Show that a solution holds a chloride.",
        hint: "Silver nitrate on salt water. A white curd means chloride. Try it on \
               potassium iodide too and watch the colour change.",
        done: |v| v.amount("AgCl") > 1.0,
    },
    Lesson {
        title: "Golden rain",
        goal: "Make bright yellow flakes fall through the water.",
        hint: "Lead nitrate and potassium iodide. Both solutions are clear, and what \
               falls out of them is not.",
        done: |v| v.amount("PbI2") > 1.0,
    },
    Lesson {
        title: "Chalk out of thin air",
        goal: "Turn limewater milky with a gas you made yourself.",
        hint: "Put limewater in the glass, then baking soda and an acid. The fizz is \
               carbon dioxide, and the cloud it leaves is chalk.",
        done: |v| v.seen.contains("CO2") && v.amount("CaCO3") > 0.5,
    },
    Lesson {
        title: "A silver tree",
        goal: "Grow silver on a strip of copper.",
        hint: "Copper in silver nitrate, in a beaker so you can see it. The copper \
               takes the nitrate and hands over its electrons; the silver has nowhere \
               to go but out. It takes a minute.",
        done: |v| v.amount("Ag") > 3.0,
    },
    Lesson {
        title: "The iodine clock",
        goal: "Mix a glass that sits clear for a while and then snaps blue.",
        hint: "In a beaker: water, hydrogen peroxide, potassium iodide, sulfuric acid, \
               starch, and one measure of hypo. The hypo eats the iodine as fast as it \
               is made. When the hypo runs out, the starch gets it all at once.",
        done: |v| v.blue() > 0.5 && v.seen.contains("Na2S2O3") && v.seen.contains("H2O2"),
    },
    Lesson {
        title: "The clock that keeps going",
        goal: "Make one that swings blue and clear and blue again, four times over.",
        hint: "In a beaker: water, potassium iodate, hydrogen peroxide (plenty), malonic \
               acid, manganese sulfate, sulfuric acid and starch. Iodous acid makes more \
               of itself, and that is what makes it swing instead of settle.",
        done: |v| v.flips >= 4,
    },
    Lesson {
        title: "Flame colours",
        goal: "Find the salt that burns blue green.",
        hint: "Dissolve a salt, then dip the wire: the t key. Every metal has its own \
               colour, which is how we know what the stars are made of.",
        done: |v| { let f = v.flame(); matches!(f, Some(((_, g, b), _)) if g > 150 && b > 120) },
    },
];

/// Which experiments have been passed, kept in `~/.alchemy/done`.
pub fn load_done(path: &std::path::Path) -> Vec<bool> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    (0..=LESSONS.len()).map(|i| text.split_whitespace().any(|d| d == i.to_string())).collect()
}

pub fn save_done(path: &std::path::Path, done: &[bool]) {
    let list: Vec<String> = done.iter().enumerate().filter(|(_, &d)| d).map(|(i, _)| i.to_string()).collect();
    let _ = std::fs::write(path, list.join(" "));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chem::{Glass, Vessel, PER_ML};

    fn run(v: &mut Vessel, seconds: f64) {
        for _ in 0..(seconds / 0.05) as usize { v.step(0.05); }
    }

    #[test]
    fn every_experiment_can_be_passed() {
        // One vessel per experiment, set up the way the hint describes.
        let mut wins: Vec<Vessel> = Vec::new();

        let mut a = Vessel::new(Glass::Tube);
        a.add("H2O", 10.0 * PER_ML); a.add("CuSO4", 10.0);
        wins.push(a);

        let mut b = Vessel::new(Glass::Tube);
        b.add("H2O", 10.0 * PER_ML); b.add("starch", 2.0); b.add("I2", 5.0);
        wins.push(b);

        let mut c = Vessel::new(Glass::Tube);
        c.add("H2O", 10.0 * PER_ML); c.add("HCl", 10.0); c.add("NaOH", 10.0);
        run(&mut c, 5.0); c.burner = true; run(&mut c, 60.0);
        wins.push(c);

        let mut d = Vessel::new(Glass::Tube);
        d.add("H2O", 10.0 * PER_ML); d.add("NaCl", 10.0); d.add("AgNO3", 10.0);
        run(&mut d, 3.0);
        wins.push(d);

        let mut e = Vessel::new(Glass::Tube);
        e.add("H2O", 10.0 * PER_ML); e.add("PbNO3", 10.0); e.add("KI", 20.0);
        run(&mut e, 3.0);
        wins.push(e);

        let mut f = Vessel::new(Glass::Beaker);
        f.add("H2O", 60.0 * PER_ML); f.add("CaOH2", 5.0); f.add("NaHCO3", 10.0); f.add("HCl", 10.0);
        run(&mut f, 30.0);
        wins.push(f);

        let mut g = Vessel::new(Glass::Beaker);
        g.add("H2O", 50.0 * PER_ML); g.add("AgNO3", 20.0); g.add("Cu", 8.0);
        run(&mut g, 120.0);
        wins.push(g);

        let mut h = Vessel::new(Glass::Beaker);
        h.add("H2O", 100.0 * PER_ML); h.add("H2O2", 40.0); h.add("KI", 20.0);
        h.add("H2SO4", 10.0); h.add("starch", 5.0); h.add("Na2S2O3", 4.0);
        run(&mut h, 120.0);
        wins.push(h);

        let mut i = Vessel::new(Glass::Beaker);
        i.add("H2O", 100.0 * PER_ML); i.add("KIO3", 60.0); i.add("H2O2", 400.0);
        i.add("MA", 200.0); i.add("MnSO4", 2.0); i.add("H2SO4", 20.0); i.add("starch", 5.0);
        run(&mut i, 200.0);
        wins.push(i);

        let mut j = Vessel::new(Glass::Tube);
        j.add("H2O", 10.0 * PER_ML); j.add("CuSO4", 10.0);
        wins.push(j);

        assert_eq!(wins.len(), LESSONS.len());
        for (n, (lesson, v)) in LESSONS.iter().zip(wins.iter()).enumerate() {
            assert!((lesson.done)(v), "experiment {} ({}) cannot be passed", n + 1, lesson.title);
        }
    }

    #[test]
    fn an_empty_bench_passes_nothing() {
        let v = Vessel::new(Glass::Tube);
        for lesson in LESSONS { assert!(!(lesson.done)(&v), "{} passes on an empty glass", lesson.title); }
    }
}
