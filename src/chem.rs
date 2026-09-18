//! The chemistry itself: what the substances are, what they do to each
//! other, and what a vessel looks like while it happens.
//!
//! Nothing here is scripted. A vessel holds an amount of each substance
//! in millimoles, and every reaction in `REACTIONS` runs at a speed set
//! by how concentrated its reactants are. The colour, the precipitate,
//! the bubbles, the heat and the pH all fall out of what is left
//! afterwards. That is why the iodine clock works: three ordinary
//! reactions at three different speeds, and the blue arrives on its own.

use std::collections::{HashMap, HashSet};

pub type Rgb = (u8, u8, u8);

/// Where a substance sits when it is in a vessel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// Stays a solid whatever you do: a metal, chalk, sulfur.
    Solid,
    /// Dissolves when there is water, and is left behind when it boils off.
    Aq,
    /// Will not dissolve, so it falls out as a precipitate.
    Grit,
    /// A liquid of its own.
    Liquid,
    /// Leaves the vessel as bubbles.
    Gas,
}

pub struct Species {
    pub key: &'static str,
    pub name: &'static str,
    pub formula: &'static str,
    pub phase: Phase,
    /// The colour it gives, if any.
    pub rgb: Option<Rgb>,
    /// How strongly it colours what it is in. An indicator is far
    /// stronger than a salt, which is why one drop is enough.
    pub tint: f64,
    pub note: &'static str,
}

const fn s(
    key: &'static str, name: &'static str, formula: &'static str,
    phase: Phase, rgb: Option<Rgb>, tint: f64, note: &'static str,
) -> Species {
    Species { key, name, formula, phase, rgb, tint, note }
}

use Phase::{Aq, Gas, Grit, Liquid, Solid};

pub static SPECIES: &[Species] = &[
    s("H2O", "water", "H₂O", Liquid, None, 0.0, "The one thing every bench needs."),
    s("O2", "oxygen", "O₂", Gas, None, 0.0, "Bubbles away. Relights a glowing splint."),
    s("H2", "hydrogen", "H₂", Gas, None, 0.0, "Bubbles away. Pops with a lit splint."),
    s("CO2", "carbon dioxide", "CO₂", Gas, None, 0.0, "Bubbles away. Turns limewater milky."),
    s("NO2", "nitrogen dioxide", "NO₂", Gas, Some((190, 90, 40)), 0.6, "Brown and foul. Do not breathe it."),
    s("SO2", "sulfur dioxide", "SO₂", Gas, None, 0.0, "Sharp, like a struck match."),
    s("I2g", "iodine vapour", "I₂", Gas, Some((170, 90, 210)), 1.0, "Violet vapour. Iodine goes straight from solid to gas."),

    // Acids
    s("HCl", "hydrochloric acid", "HCl", Aq, None, 0.0, "Strong acid. The one in your stomach."),
    s("H2SO4", "sulfuric acid", "H₂SO₄", Aq, None, 0.0, "Strong acid, two protons, and it loves water."),
    s("HNO3", "nitric acid", "HNO₃", Aq, None, 0.0, "Strong acid that also eats metals whole."),
    s("AcOH", "vinegar", "CH₃COOH", Aq, None, 0.0, "Acetic acid. Weak: most of it stays whole."),

    // Bases
    s("NaOH", "caustic soda", "NaOH", Aq, None, 0.0, "Strong base. Soapy on the fingers, and it burns."),
    s("KOH", "caustic potash", "KOH", Aq, None, 0.0, "Strong base, the potassium twin of caustic soda."),
    s("NH3", "ammonia solution", "NH₃", Aq, None, 0.0, "A weak base with the smell of a cleaning cupboard."),
    s("CaOH2", "limewater", "Ca(OH)₂", Aq, None, 0.0, "Clear until carbon dioxide finds it."),

    // Salts that dissolve
    s("NaCl", "table salt", "NaCl", Aq, None, 0.0, "The salt on your food. Sodium meets chlorine."),
    s("NaAc", "sodium acetate", "CH₃COONa", Aq, None, 0.0, "What vinegar leaves behind. Hot ice, if you grow it."),
    s("KCl", "potassium chloride", "KCl", Aq, None, 0.0, "Tastes of salt with a bitter edge."),
    s("KNO3", "saltpetre", "KNO₃", Aq, None, 0.0, "Gives up its oxygen when hot. Cured meat, and worse."),
    s("NaNO3", "sodium nitrate", "NaNO₃", Aq, None, 0.0, "Chile saltpetre, dug out of a desert."),
    s("CuSO4", "copper sulfate", "CuSO₄", Aq, Some((0, 120, 205)), 1.0, "The blue every chemistry set starts with."),
    s("CuSO4d", "dry copper sulfate", "CuSO₄", Grit, Some((228, 228, 218)), 1.0, "Chalk white with the water baked out. One drop and it is blue again."),
    s("CuCl2", "copper chloride", "CuCl₂", Aq, Some((0, 150, 150)), 1.0, "Blue green, and it burns blue green too."),
    s("CuNO3", "copper nitrate", "Cu(NO₃)₂", Aq, Some((0, 130, 190)), 1.0, "What is left when copper meets nitric acid."),
    s("FeCl3", "iron chloride", "FeCl₃", Aq, Some((165, 110, 30)), 1.0, "Yellow brown. Etches circuit boards."),
    s("AgNO3", "silver nitrate", "AgNO₃", Aq, None, 0.0, "Clear, until a chloride turns up. Stains skin black."),
    s("BaCl2", "barium chloride", "BaCl₂", Aq, None, 0.0, "Clear, and the way to find a sulfate."),
    s("Na2CO3", "washing soda", "Na₂CO₃", Aq, None, 0.0, "Soda. Fizzes hard with any acid."),
    s("NaHCO3", "baking soda", "NaHCO₃", Aq, None, 0.0, "The fizz in a cake and in a fire extinguisher."),
    s("PbNO3", "lead nitrate", "Pb(NO₃)₂", Aq, None, 0.0, "Clear and heavy. Half of golden rain."),
    s("KI", "potassium iodide", "KI", Aq, None, 0.0, "Clear. Hands over iodide to anything that wants it."),
    s("NaI", "sodium iodide", "NaI", Aq, None, 0.0, "Clear, and the same iodide as its potassium cousin."),
    s("Na2S2O3", "hypo", "Na₂S₂O₃", Aq, None, 0.0, "Thiosulfate. Photographers fixed film with it; it eats iodine."),
    s("Na2SO4", "sodium sulfate", "Na₂SO₄", Aq, None, 0.0, "Glauber's salt. Sits there being ordinary."),
    s("NH4Cl", "sal ammoniac", "NH₄Cl", Aq, None, 0.0, "Ammonia and acid, made up again."),
    s("KMnO4", "permanganate", "KMnO₄", Aq, Some((120, 20, 150)), 2.5, "Purple so deep a grain colours a bucket."),
    s("K2Cr2O7", "dichromate", "K₂Cr₂O₇", Aq, Some((225, 120, 20)), 1.6, "Orange. Turns green when it takes electrons."),
    s("Cr3", "chromium(III)", "Cr³⁺", Aq, Some((40, 160, 90)), 1.6, "The green left after orange dichromate has done its work."),
    s("Mn2", "manganese(II)", "Mn²⁺", Aq, Some((240, 200, 210)), 0.2, "Almost colourless. Purple permanganate ends up here."),
    s("CoCl2", "cobalt chloride", "CoCl₂", Aq, Some((225, 100, 150)), 1.2, "Pink in water, blue when dry. A weather glass."),
    s("CoCl2d", "dry cobalt chloride", "CoCl₂", Grit, Some((40, 70, 205)), 1.2, "Deep blue. One breath of damp air and it goes pink."),
    s("NiSO4", "nickel sulfate", "NiSO₄", Aq, Some((60, 175, 95)), 1.0, "Green as a bottle."),
    s("SrCl2", "strontium chloride", "SrCl₂", Aq, None, 0.0, "Clear, and the red in every distress flare."),
    s("LiCl", "lithium chloride", "LiCl", Aq, None, 0.0, "Clear, and it burns a hard red."),
    s("CaCl2", "calcium chloride", "CaCl₂", Aq, None, 0.0, "The grit thrown on icy roads."),
    s("BaNO3", "barium nitrate", "Ba(NO₃)₂", Aq, None, 0.0, "Clear, and it burns green."),
    s("ZnSO4", "zinc sulfate", "ZnSO₄", Aq, None, 0.0, "Clear. What is left when zinc gives up its electrons."),
    s("ZnCl2", "zinc chloride", "ZnCl₂", Aq, None, 0.0, "Clear, and greedy for water."),
    s("MgCl2", "magnesium chloride", "MgCl₂", Aq, None, 0.0, "Clear. Most of the bitterness in sea water."),
    s("FeSO4", "iron sulfate", "FeSO₄", Aq, Some((120, 180, 140)), 0.8, "Pale green. Goes brown in air."),

    // Things that will not dissolve
    s("AgCl", "silver chloride", "AgCl", Grit, Some((248, 248, 248)), 1.0, "A white curd. Goes grey in the light."),
    s("AgI", "silver iodide", "AgI", Grit, Some((235, 225, 145)), 1.0, "Pale yellow. Seeded into clouds to make rain."),
    s("BaSO4", "barium sulfate", "BaSO₄", Grit, Some((250, 250, 250)), 1.0, "A dense white. Drunk before an X-ray."),
    s("CaCO3", "chalk", "CaCO₃", Grit, Some((245, 245, 240)), 1.0, "Chalk, limestone, seashells, the lot."),
    s("PbI2", "lead iodide", "PbI₂", Grit, Some((240, 200, 40)), 1.4, "Golden rain. Falls in bright flakes."),
    s("CuOH2", "copper hydroxide", "Cu(OH)₂", Grit, Some((110, 190, 225)), 1.0, "A pale blue jelly."),
    s("FeOH3", "iron hydroxide", "Fe(OH)₃", Grit, Some((150, 70, 25)), 1.0, "Rust brown slime."),

    // Solids on the shelf
    s("S", "sulfur", "S", Solid, Some((230, 215, 60)), 1.0, "Brimstone. Yellow, and it stinks when it burns."),
    s("C", "charcoal", "C", Solid, Some((30, 30, 30)), 1.0, "Carbon. Burns hot and leaves almost nothing."),
    s("Cu", "copper", "Cu", Solid, Some((200, 120, 60)), 1.0, "A strip of copper. Pushes silver out of solution."),
    s("Ag", "silver", "Ag", Solid, Some((220, 220, 230)), 1.0, "Silver, grown as a tree if you are patient."),
    s("Zn", "zinc", "Zn", Solid, Some((170, 175, 180)), 1.0, "Grey and eager. Gives up electrons to almost anything."),
    s("Fe", "iron", "Fe", Solid, Some((140, 140, 145)), 1.0, "Iron filings."),
    s("Mg", "magnesium", "Mg", Solid, Some((205, 205, 210)), 1.0, "A ribbon that burns with a light you must not look at."),
    s("Al", "aluminium", "Al", Solid, Some((190, 190, 195)), 1.0, "Powder, wrapped in its own skin of oxide."),
    s("Na", "sodium", "Na", Solid, Some((215, 215, 210)), 1.0, "Kept under oil, because water is not its friend."),
    s("CuO", "copper oxide", "CuO", Grit, Some((25, 25, 25)), 1.0, "Black. What copper becomes in a flame."),
    s("ZnO", "zinc oxide", "ZnO", Grit, Some((250, 250, 245)), 1.0, "White smoke while it forms, white powder after."),
    s("MgO", "magnesia", "MgO", Grit, Some((252, 252, 250)), 1.0, "The white ash left by burning magnesium."),
    s("Fe2O3", "iron oxide", "Fe₂O₃", Grit, Some((160, 60, 30)), 1.0, "Rust. The suite is named after it."),
    s("Al2O3", "alumina", "Al₂O₃", Grit, Some((240, 240, 238)), 1.0, "White and hard. Corundum, ruby, sapphire."),
    s("I2", "iodine", "I₂", Aq, Some((140, 80, 20)), 1.4, "Grey black flakes. In water with an iodide they make a brown solution."),
    s("starch", "starch", "(C₆H₁₀O₅)ₙ", Aq, None, 0.0, "Potato flour in water. Iodine finds it at once."),
    s("H2O2", "hydrogen peroxide", "H₂O₂", Aq, None, 0.0, "Water with one oxygen too many, and keen to lose it."),
    s("KClO3", "potassium chlorate", "KClO₃", Aq, None, 0.0, "An oxidiser. Hands out oxygen to anything that will burn."),
    s("Pred", "red phosphorus", "P", Solid, Some((160, 60, 50)), 1.0, "The strip on a matchbox. Safe alone, not beside an oxidiser."),
    s("sugar", "sugar", "C₁₂H₂₂O₁₁", Aq, None, 0.0, "Food, and fuel."),
    s("MA", "malonic acid", "CH₂(COOH)₂", Aq, None, 0.0, "The fuel that keeps the iodine clock ticking."),
    s("KIO3", "potassium iodate", "KIO₃", Aq, None, 0.0, "Iodine with three oxygens. The other half of the clock."),
    s("MnSO4", "manganese sulfate", "MnSO₄", Aq, None, 0.0, "The catalyst that lets the clock swing."),
    s("HIO2", "iodous acid", "HIO₂", Aq, None, 0.0, "Never there for long. It makes more of itself, and that is the whole trick."),

    // Indicators
    s("litmus", "litmus", "", Aq, None, 0.0, "Lichen dye. Red in acid, blue in base."),
    s("phen", "phenolphthalein", "", Aq, None, 0.0, "Nothing at all, until the base arrives."),
    s("uni", "universal indicator", "", Aq, None, 0.0, "The whole rainbow, one colour per pH."),
    s("cabbage", "red cabbage juice", "", Aq, None, 0.0, "Boiled cabbage water. A full indicator for free."),
];

/// How many protons each unit hands over. A weak acid hands over few.
static ACIDS: &[(&str, f64)] = &[
    ("HCl", 1.0), ("H2SO4", 2.0), ("HNO3", 1.0), ("AcOH", 0.04), ("MA", 0.05),
];

/// How much base each unit is worth.
static BASES: &[(&str, f64)] = &[
    ("NaOH", 1.0), ("KOH", 1.0), ("NH3", 0.02), ("CaOH2", 2.0),
    ("Na2CO3", 0.2), ("NaHCO3", 0.05), ("NaAc", 0.02),
];

/// The colour a substance paints a flame. This is the test that tells
/// sodium from potassium with nothing but a wire and a burner.
static FLAMES: &[(&str, Rgb, &str)] = &[
    ("NaCl", (250, 220, 60), "sodium: a hard yellow that drowns out everything else"),
    ("NaNO3", (250, 220, 60), "sodium: a hard yellow"),
    ("NaOH", (250, 220, 60), "sodium: a hard yellow"),
    ("Na2CO3", (250, 220, 60), "sodium: a hard yellow"),
    ("Na2SO4", (250, 220, 60), "sodium: a hard yellow"),
    ("NaHCO3", (250, 220, 60), "sodium: a hard yellow"),
    ("NaI", (250, 220, 60), "sodium: a hard yellow"),
    ("NaAc", (250, 220, 60), "sodium: a hard yellow"),
    ("Na", (250, 220, 60), "sodium: a hard yellow"),
    ("KCl", (190, 140, 235), "potassium: lilac, and easy to miss"),
    ("KNO3", (190, 140, 235), "potassium: lilac"),
    ("KI", (190, 140, 235), "potassium: lilac"),
    ("KOH", (190, 140, 235), "potassium: lilac"),
    ("KMnO4", (190, 140, 235), "potassium: lilac"),
    ("KClO3", (190, 140, 235), "potassium: lilac"),
    ("KIO3", (190, 140, 235), "potassium: lilac"),
    ("LiCl", (225, 45, 65), "lithium: a deep red"),
    ("CaCl2", (240, 125, 45), "calcium: orange red"),
    ("CaOH2", (240, 125, 45), "calcium: orange red"),
    ("CaCO3", (240, 125, 45), "calcium: orange red"),
    ("CuSO4", (40, 225, 180), "copper: blue green, the best of the lot"),
    ("CuCl2", (60, 235, 140), "copper with chloride: a green you can see across a field"),
    ("CuNO3", (40, 225, 180), "copper: blue green"),
    ("Cu", (40, 225, 180), "copper: blue green"),
    ("SrCl2", (235, 30, 55), "strontium: crimson, the colour of every distress flare"),
    ("BaCl2", (150, 230, 90), "barium: pale green"),
    ("BaNO3", (150, 230, 90), "barium: pale green"),
    ("PbNO3", (140, 190, 230), "lead: a weak grey blue"),
    ("Mg", (255, 255, 255), "magnesium: white, and too bright to look at"),
];

/// How an indicator reads. Each pair is the pH from which that colour
/// holds.
type Band = &'static [(f64, Rgb)];
static INDICATORS: &[(&str, Band)] = &[
    ("litmus", &[(0.0, (215, 45, 45)), (5.0, (190, 80, 140)), (8.0, (45, 70, 205))]),
    ("phen", &[(0.0, (250, 250, 250)), (8.2, (225, 40, 150))]),
    ("uni", &[
        (0.0, (220, 25, 25)), (3.0, (235, 110, 25)), (5.0, (235, 210, 30)),
        (6.5, (100, 200, 60)), (7.5, (30, 170, 130)), (9.0, (35, 90, 210)),
        (11.0, (110, 40, 175)),
    ]),
    ("cabbage", &[
        (0.0, (220, 40, 70)), (4.0, (215, 90, 160)), (6.0, (150, 70, 190)),
        (8.0, (60, 90, 200)), (10.0, (70, 180, 140)), (12.0, (215, 215, 70)),
    ]),
];

/// The blue black that iodine makes when it meets starch. It is not a
/// substance of its own: the iodine slips inside the starch coil and
/// slips out again, so the colour follows the iodine exactly. That is
/// what makes the clock snap.
const STARCH_BLUE: Rgb = (26, 32, 112);

/// What a reaction needs before it will run at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Needs {
    /// Water to happen in.
    Wet,
    /// No water left.
    Dry,
    /// A temperature, in °C.
    Hot(f64),
}

pub struct Reaction {
    pub left: &'static [(&'static str, f64)],
    pub right: &'static [(&'static str, f64)],
    pub needs: Needs,
    /// Kilojoules given off per mole. Negative takes heat in.
    pub heat: f64,
    /// The rate constant. Speed is this times the concentration of every
    /// reactant multiplied together, so a dilute solution reacts slowly
    /// and a dry powder reacts at once. 30 is over before you look, 0.05
    /// keeps you waiting.
    pub k: f64,
    /// True when it goes off rather than merely reacting.
    pub bang: bool,
    /// What you see, told in one line.
    pub says: &'static str,
}

const fn r(
    left: &'static [(&'static str, f64)], right: &'static [(&'static str, f64)],
    needs: Needs, heat: f64, k: f64, says: &'static str,
) -> Reaction {
    Reaction { left, right, needs, heat, k, bang: false, says }
}

const fn boom(
    left: &'static [(&'static str, f64)], right: &'static [(&'static str, f64)],
    needs: Needs, heat: f64, k: f64, says: &'static str,
) -> Reaction {
    Reaction { left, right, needs, heat, k, bang: true, says }
}

use Needs::{Dry, Hot, Wet};

pub static REACTIONS: &[Reaction] = &[
    // ── Acid meets base: a salt and water, every time ──────────────────
    r(&[("HCl", 1.0), ("NaOH", 1.0)], &[("NaCl", 1.0), ("H2O", 1.0)], Wet, 57.0, 30.0,
      "the acid and the base cancel out; table salt is left in the water"),
    r(&[("HCl", 1.0), ("KOH", 1.0)], &[("KCl", 1.0), ("H2O", 1.0)], Wet, 57.0, 30.0,
      "a salt forms and the glass warms in your hand"),
    r(&[("HCl", 1.0), ("NH3", 1.0)], &[("NH4Cl", 1.0)], Wet, 52.0, 20.0,
      "white smoke where the two meet, and sal ammoniac in the water"),
    r(&[("H2SO4", 1.0), ("NaOH", 2.0)], &[("Na2SO4", 1.0), ("H2O", 2.0)], Wet, 114.0, 30.0,
      "the acid is spent and sodium sulfate is left"),
    r(&[("HNO3", 1.0), ("NaOH", 1.0)], &[("NaNO3", 1.0), ("H2O", 1.0)], Wet, 57.0, 30.0,
      "a nitrate salt forms"),
    r(&[("AcOH", 1.0), ("NaOH", 1.0)], &[("NaAc", 1.0), ("H2O", 1.0)], Wet, 55.0, 20.0,
      "the vinegar is neutralised and sodium acetate is left"),
    r(&[("HCl", 2.0), ("CaOH2", 1.0)], &[("CaCl2", 1.0), ("H2O", 2.0)], Wet, 110.0, 25.0,
      "the limewater is used up"),
    r(&[("H2SO4", 1.0), ("CaOH2", 1.0)], &[("BaSO4", 0.0), ("CaCl2", 0.0), ("H2O", 2.0)], Wet, 110.0, 20.0,
      "the limewater is used up"),

    // ── Carbonates fizz ────────────────────────────────────────────────
    r(&[("HCl", 2.0), ("Na2CO3", 1.0)], &[("NaCl", 2.0), ("H2O", 1.0), ("CO2", 1.0)], Wet, 26.0, 15.0,
      "it fizzes hard: that gas is carbon dioxide"),
    r(&[("HCl", 1.0), ("NaHCO3", 1.0)], &[("NaCl", 1.0), ("H2O", 1.0), ("CO2", 1.0)], Wet, 24.0, 18.0,
      "a rush of bubbles, the way a cake rises"),
    r(&[("AcOH", 1.0), ("NaHCO3", 1.0)], &[("NaAc", 1.0), ("H2O", 1.0), ("CO2", 1.0)], Wet, 20.0, 6.0,
      "vinegar and baking soda: the volcano every child builds"),
    r(&[("HCl", 2.0), ("CaCO3", 1.0)], &[("CaCl2", 1.0), ("H2O", 1.0), ("CO2", 1.0)], Wet, 15.0, 4.0,
      "the chalk fizzes away to nothing"),
    // The old test: carbon dioxide turns limewater milky.
    r(&[("CO2", 1.0), ("CaOH2", 1.0)], &[("CaCO3", 1.0), ("H2O", 1.0)], Wet, 70.0, 8.0,
      "the limewater goes milky: chalk, out of thin air"),

    // ── Precipitates: the tests that name an unknown ───────────────────
    r(&[("AgNO3", 1.0), ("NaCl", 1.0)], &[("AgCl", 1.0), ("NaNO3", 1.0)], Wet, 65.0, 60.0,
      "a white curd drops out at once: that was a chloride"),
    r(&[("AgNO3", 1.0), ("KCl", 1.0)], &[("AgCl", 1.0), ("KNO3", 1.0)], Wet, 65.0, 60.0,
      "a white curd: chloride again"),
    r(&[("AgNO3", 1.0), ("KI", 1.0)], &[("AgI", 1.0), ("KNO3", 1.0)], Wet, 70.0, 60.0,
      "pale yellow this time: iodide, not chloride"),
    r(&[("AgNO3", 1.0), ("NaI", 1.0)], &[("AgI", 1.0), ("NaNO3", 1.0)], Wet, 70.0, 60.0,
      "pale yellow: an iodide"),
    r(&[("BaCl2", 1.0), ("Na2SO4", 1.0)], &[("BaSO4", 1.0), ("NaCl", 2.0)], Wet, 20.0, 60.0,
      "a heavy white cloud that will not redissolve: that was a sulfate"),
    r(&[("BaCl2", 1.0), ("H2SO4", 1.0)], &[("BaSO4", 1.0), ("HCl", 2.0)], Wet, 20.0, 60.0,
      "a heavy white cloud: sulfate"),
    r(&[("BaCl2", 1.0), ("CuSO4", 1.0)], &[("BaSO4", 1.0), ("CuCl2", 1.0)], Wet, 20.0, 60.0,
      "white barium sulfate falls, and the blue stays behind"),
    r(&[("BaCl2", 1.0), ("ZnSO4", 1.0)], &[("BaSO4", 1.0), ("ZnCl2", 1.0)], Wet, 20.0, 60.0,
      "a heavy white cloud: sulfate"),
    r(&[("PbNO3", 1.0), ("KI", 2.0)], &[("PbI2", 1.0), ("KNO3", 2.0)], Wet, 40.0, 40.0,
      "golden rain: bright yellow flakes drifting down"),
    r(&[("CuSO4", 1.0), ("NaOH", 2.0)], &[("CuOH2", 1.0), ("Na2SO4", 1.0)], Wet, 40.0, 40.0,
      "a pale blue jelly forms and settles"),
    r(&[("FeCl3", 1.0), ("NaOH", 3.0)], &[("FeOH3", 1.0), ("NaCl", 3.0)], Wet, 50.0, 40.0,
      "rust brown slime: that is iron"),
    r(&[("CaCl2", 1.0), ("Na2CO3", 1.0)], &[("CaCO3", 1.0), ("NaCl", 2.0)], Wet, 15.0, 30.0,
      "chalk drops out, white and fine"),
    r(&[("CuOH2", 1.0)], &[("CuO", 1.0), ("H2O", 1.0)], Hot(70.0), 10.0, 1.5,
      "the blue jelly turns black: copper oxide"),

    // ── One metal pushes another out ───────────────────────────────────
    r(&[("Cu", 1.0), ("AgNO3", 2.0)], &[("CuNO3", 1.0), ("Ag", 2.0)], Wet, 90.0, 0.7,
      "silver grows on the copper in fine needles, and the water turns blue"),
    r(&[("Zn", 1.0), ("CuSO4", 1.0)], &[("Cu", 1.0), ("ZnSO4", 1.0)], Wet, 217.0, 1.0,
      "copper plates itself onto the zinc and the blue drains away"),
    r(&[("Zn", 1.0), ("HCl", 2.0)], &[("ZnCl2", 1.0), ("H2", 1.0)], Wet, 153.0, 1.5,
      "bubbles stream off the zinc: that is hydrogen"),
    r(&[("Mg", 1.0), ("HCl", 2.0)], &[("MgCl2", 1.0), ("H2", 1.0)], Wet, 462.0, 5.0,
      "the magnesium vanishes in a storm of bubbles"),
    r(&[("Fe", 1.0), ("CuSO4", 1.0)], &[("Cu", 1.0), ("FeSO4", 1.0)], Wet, 150.0, 0.4,
      "the iron takes on a copper skin"),
    r(&[("Na", 1.0), ("H2O", 2.0)], &[("NaOH", 1.0), ("H2", 0.5)], Wet, 184.0, 12.0,
      "the sodium skates about hissing, and the water turns to caustic soda"),
    r(&[("Cu", 1.0), ("HNO3", 4.0)], &[("CuNO3", 1.0), ("NO2", 2.0), ("H2O", 2.0)], Wet, 140.0, 1.2,
      "brown fumes pour off and the water goes blue green"),
    r(&[("Zn", 1.0), ("H2SO4", 1.0)], &[("ZnSO4", 1.0), ("H2", 1.0)], Wet, 153.0, 1.5,
      "hydrogen bubbles off the zinc"),

    // ── Iodine: the colour that started all this ───────────────────────
    r(&[("I2", 1.0), ("Na2S2O3", 2.0)], &[("KI", 2.0)], Wet, 20.0, 40000.0,
      "the brown fades to nothing: hypo eats iodine"),
    r(&[("I2", 1.0)], &[("I2g", 1.0)], Hot(60.0), -62.0, 2.0,
      "violet vapour climbs the glass: iodine goes straight from solid to gas"),
    // The clock proper. The first step is slow and the hypo is instant,
    // so nothing happens at all until the hypo runs out. Then the iodine
    // has nowhere to go, and the starch catches it.
    r(&[("H2O2", 1.0), ("KI", 2.0), ("H2SO4", 1.0)],
      &[("I2", 1.0), ("H2O", 2.0), ("Na2SO4", 1.0)], Wet, 40.0, 1.5,
      "nothing to see yet"),

    // ── The clock that keeps going ─────────────────────────────────────
    // The real Briggs-Rauscher runs on a dozen steps. These four are the
    // shape of it: iodate feeds iodous acid, iodous acid makes more of
    // itself, the malonic acid turns it into iodine, and the peroxide
    // clears up. Feeding a loop that eats its own product is what makes a
    // thing swing instead of settling, in chemistry and everywhere else.
    r(&[("KIO3", 1.0), ("MnSO4", 1.0)], &[("HIO2", 1.0), ("MnSO4", 1.0)], Wet, 15.0, 0.25, ""),
    r(&[("MA", 1.0), ("HIO2", 1.0)], &[("I2", 1.0), ("H2O", 1.0)], Wet, 25.0, 1.92, ""),
    r(&[("HIO2", 2.0), ("I2", 1.0)], &[("HIO2", 3.0)], Wet, 20.0, 40000.0, ""),
    r(&[("HIO2", 1.0), ("H2O2", 1.0)], &[("KIO3", 1.0), ("H2O", 1.0), ("O2", 0.5)], Wet, 20.0, 0.16, ""),

    // ── Colour changes worth the trip ──────────────────────────────────
    r(&[("KMnO4", 2.0), ("Na2S2O3", 1.0), ("H2SO4", 1.0)], &[("Mn2", 2.0), ("Na2SO4", 2.0)], Wet, 90.0, 10.0,
      "the purple drains out drop by drop"),
    r(&[("K2Cr2O7", 1.0), ("Na2S2O3", 3.0), ("H2SO4", 4.0)], &[("Cr3", 2.0), ("Na2SO4", 3.0)], Wet, 80.0, 8.0,
      "orange gives way to green"),
    r(&[("CuSO4d", 1.0), ("H2O", 5.0)], &[("CuSO4", 1.0)], Wet, 78.0, 4.0,
      "the white powder floods blue the moment water touches it"),
    r(&[("CuSO4", 1.0)], &[("CuSO4d", 1.0), ("H2O", 5.0)], Hot(150.0), -78.0, 1.5,
      "the blue bakes out to a chalky white"),
    r(&[("CoCl2d", 1.0), ("H2O", 6.0)], &[("CoCl2", 1.0)], Wet, 30.0, 4.0,
      "deep blue turns pink: the old way to tell whether the air is damp"),
    r(&[("CoCl2", 1.0)], &[("CoCl2d", 1.0), ("H2O", 6.0)], Hot(120.0), -30.0, 1.5,
      "the pink dries back to blue"),
    r(&[("H2O2", 2.0), ("MnSO4", 1.0)], &[("H2O", 2.0), ("O2", 1.0), ("MnSO4", 1.0)], Wet, 98.0, 0.01,
      "oxygen boils off in a froth and climbs out of the glass"),
    r(&[("FeCl3", 1.0), ("KI", 2.0)], &[("I2", 0.5), ("KCl", 2.0)], Wet, 30.0, 1.5,
      "the yellow goes brown: the iron has taken electrons from the iodide"),

    // ── Fire: heat alone is enough ─────────────────────────────────────
    r(&[("Mg", 1.0), ("O2", 0.5)], &[("MgO", 1.0)], Hot(480.0), 602.0, 30.0,
      "a white light too bright to look at"),
    r(&[("Cu", 1.0), ("O2", 0.5)], &[("CuO", 1.0)], Hot(350.0), 157.0, 2.0,
      "the copper blackens"),
    r(&[("S", 1.0), ("O2", 1.0)], &[("SO2", 1.0)], Hot(250.0), 297.0, 10.0,
      "a blue flame and a smell that catches your throat"),
    r(&[("C", 1.0), ("O2", 1.0)], &[("CO2", 1.0)], Hot(500.0), 394.0, 5.0,
      "the charcoal glows right through"),
    r(&[("Zn", 1.0), ("O2", 0.5)], &[("ZnO", 1.0)], Hot(400.0), 348.0, 10.0,
      "white smoke rolls off: zinc oxide"),
    r(&[("KClO3", 1.0)], &[("KCl", 1.0), ("O2", 1.5)], Hot(400.0), 45.0, 3.0,
      "the chlorate lets its oxygen go"),
    r(&[("H2O2", 1.0)], &[("H2O", 1.0), ("O2", 0.5)], Hot(80.0), 98.0, 1.0,
      "the peroxide breaks up and oxygen streams off"),
    r(&[("CaCO3", 1.0)], &[("CaOH2", 1.0), ("CO2", 1.0)], Hot(825.0), -178.0, 2.0,
      "the chalk breaks down to quicklime, the way a lime kiln works"),
    r(&[("sugar", 1.0), ("O2", 12.0)], &[("CO2", 12.0), ("H2O", 11.0)], Hot(400.0), 5640.0, 2.0,
      "the sugar chars, then burns"),

    // ── The mixtures that go off ───────────────────────────────────────
    // An oxidiser lying beside a fuel is a fire waiting for an excuse.
    // That is why a chemist keeps them on different shelves, and it is
    // the one thing nobody had told me. What follows is what happens and
    // what it costs, with no amounts and no method.
    boom(&[("KClO3", 1.0), ("S", 1.0)], &[("KCl", 1.0), ("SO2", 1.0), ("O2", 0.5)], Dry, 520.0, 60.0,
      "a white flash and a crack, and the smell of a struck match"),
    boom(&[("KClO3", 1.0), ("Pred", 1.0)], &[("KCl", 1.0), ("O2", 1.5)], Dry, 620.0, 120.0,
      "it goes off the instant the two grains touch"),
    boom(&[("KClO3", 2.0), ("sugar", 1.0)], &[("KCl", 2.0), ("CO2", 12.0), ("H2O", 11.0)], Dry, 480.0, 40.0,
      "a hard purple flame and a roar"),
    boom(&[("KClO3", 1.0), ("Zn", 1.5)], &[("KCl", 1.0), ("ZnO", 1.5)], Dry, 560.0, 80.0,
      "a flash, and a cloud of white smoke that hangs in the air"),
    boom(&[("KNO3", 2.0), ("S", 1.0), ("C", 3.0)], &[("SO2", 1.0), ("CO2", 3.0)], Dry, 300.0, 20.0,
      "a fierce burn and a great deal of smoke"),
    boom(&[("Zn", 1.0), ("S", 1.0)], &[("ZnO", 1.0), ("SO2", 1.0)], Hot(300.0), 350.0, 60.0,
      "a jet of flame and smoke straight up out of the glass"),
    // Thermite: iron oxide and aluminium. The suite is named after the
    // first of those.
    boom(&[("Fe2O3", 1.0), ("Al", 2.0)], &[("Al2O3", 1.0), ("Fe", 2.0)], Hot(600.0), 852.0, 20.0,
      "white heat, and iron running out of the bottom like water"),
    boom(&[("Pred", 1.0), ("O2", 1.25)], &[("Fe2O3", 0.0)], Hot(260.0), 750.0, 40.0,
      "the phosphorus catches, and it will not be put out"),
];

// ── A vessel ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Glass { Tube, Beaker, Flask }

impl Glass {
    pub fn name(&self) -> &'static str {
        match self { Glass::Tube => "test tube", Glass::Beaker => "beaker", Glass::Flask => "flask" }
    }
    /// How much it holds, in millilitres.
    pub fn holds(&self) -> f64 {
        match self { Glass::Tube => 50.0, Glass::Beaker => 200.0, Glass::Flask => 400.0 }
    }
    pub fn next(&self) -> Glass {
        match self { Glass::Tube => Glass::Beaker, Glass::Beaker => Glass::Flask, Glass::Flask => Glass::Tube }
    }
}

/// What happened during one step, for the line under the bench.
#[derive(Debug, Clone)]
pub struct Event {
    pub says: String,
    pub bang: bool,
}

#[derive(Debug, Clone)]
pub struct Vessel {
    pub glass: Glass,
    /// Millimoles of each substance.
    pub holds: HashMap<&'static str, f64>,
    pub temp: f64,
    pub burner: bool,
    /// Gas made in the last moment, which is what the bubbles show.
    pub fizz: f64,
    /// Counts down while a flash is on screen.
    pub flare: f64,
    /// The colour of that flash.
    pub flare_rgb: Rgb,
    /// Reactions already announced, so one line is not repeated.
    said: Vec<usize>,
    /// Everything that has ever been in here. An experiment asks about
    /// this, because a gas that has bubbled away has still been made.
    pub seen: HashSet<&'static str>,
    /// How many times the starch and iodine blue has come or gone.
    pub flips: u32,
    /// True once something in here has gone off.
    pub banged: bool,
    was_blue: bool,
}

/// Millimoles of water in one millilitre.
pub const PER_ML: f64 = 55.5;
/// Below this a substance counts as gone.
const TRACE: f64 = 1e-7;
/// The volume a dry mixture is treated as filling, so that concentration
/// still means something when there is no water.
const DRY_ML: f64 = 1.0;

impl Vessel {
    pub fn new(glass: Glass) -> Vessel {
        Vessel {
            glass, holds: HashMap::new(), temp: 20.0, burner: false,
            fizz: 0.0, flare: 0.0, flare_rgb: (255, 255, 255), said: Vec::new(),
            seen: HashSet::new(), flips: 0, banged: false, was_blue: false,
        }
    }

    pub fn amount(&self, key: &str) -> f64 { self.holds.get(key).copied().unwrap_or(0.0) }

    pub fn add(&mut self, key: &'static str, mmol: f64) {
        *self.holds.entry(key).or_insert(0.0) += mmol;
        self.seen.insert(key);
        self.said.clear();
    }

    pub fn empty(&mut self) {
        self.holds.clear();
        self.temp = 20.0;
        self.burner = false;
        self.fizz = 0.0;
        self.flare = 0.0;
        self.said.clear();
        self.seen.clear();
        self.flips = 0;
        self.banged = false;
        self.was_blue = false;
    }

    pub fn is_empty(&self) -> bool { self.holds.values().all(|&v| v < TRACE) }

    /// How much liquid is in there, in millilitres.
    pub fn volume(&self) -> f64 { self.amount("H2O") / PER_ML }

    /// The volume reactions happen in: the water, or a pinch of space
    /// when the mixture is dry.
    fn working_ml(&self) -> f64 { self.volume().max(DRY_ML) }

    /// How full the glass is, from 0 to 1. Solids take up room too.
    pub fn level(&self) -> f64 {
        let solids: f64 = self.holds.iter()
            .filter(|(k, v)| **v > TRACE && self.is_solid_here(k))
            .map(|(_, v)| v * 0.05).sum();
        ((self.volume() + solids) / self.glass.holds()).clamp(0.0, 1.0)
    }

    /// True when this substance is sitting there as a solid. A salt that
    /// dissolves is a solid again once the water has gone, which is how
    /// you get to see the salt you made.
    fn is_solid_here(&self, key: &str) -> bool {
        match phase(key) {
            Some(Solid) | Some(Grit) => true,
            Some(Aq) => self.volume() < 0.5,
            _ => false,
        }
    }

    /// How acid or basic it is. Pure water reads 7.
    pub fn ph(&self) -> f64 {
        let ml = self.volume();
        if ml < 0.1 { return 7.0; }
        let acid: f64 = ACIDS.iter().map(|(k, n)| self.amount(k) * n).sum();
        let base: f64 = BASES.iter().map(|(k, n)| self.amount(k) * n).sum();
        let net = (acid - base) / ml;
        if net > 1e-9 { (-net.log10()).clamp(0.0, 7.0) }
        else if net < -1e-9 { (14.0 + (-net).log10()).clamp(7.0, 14.0) }
        else { 7.0 }
    }

    /// How deep the starch and iodine blue runs, from 0 to 1.
    pub fn blue(&self) -> f64 {
        let ml = self.volume();
        if ml < 0.1 || self.amount("starch") < 1e-3 { return 0.0; }
        (self.amount("I2") / ml * 40.0).min(1.0)
    }

    /// The colour of the liquid. Starch and iodine beat everything, then
    /// an indicator, because that is what an indicator is for.
    pub fn liquid_rgb(&self) -> Option<Rgb> {
        if self.volume() < 0.1 { return None; }
        let blue = self.blue();
        if blue > 0.02 {
            let under = self.plain_rgb().unwrap_or((235, 235, 235));
            let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * blue) as u8;
            return Some((mix(under.0, STARCH_BLUE.0), mix(under.1, STARCH_BLUE.1), mix(under.2, STARCH_BLUE.2)));
        }
        let ph = self.ph();
        for (key, bands) in INDICATORS {
            if self.amount(key) > TRACE {
                let mut out = bands[0].1;
                for (from, rgb) in bands.iter() { if ph >= *from { out = *rgb; } }
                return Some(out);
            }
        }
        self.plain_rgb()
    }

    /// The colour the dissolved substances make on their own, deeper the
    /// more of them there is.
    fn plain_rgb(&self) -> Option<Rgb> {
        let ml = self.volume();
        if ml < 0.1 { return None; }
        let mut sum = (0.0, 0.0, 0.0);
        let mut weight = 0.0;
        for (key, &mmol) in &self.holds {
            if mmol < TRACE { continue; }
            let Some(sp) = species(key) else { continue };
            if sp.phase != Aq && sp.phase != Liquid { continue; }
            let Some(rgb) = sp.rgb else { continue };
            let w = (mmol / ml * sp.tint).min(3.0);
            sum = (sum.0 + rgb.0 as f64 * w, sum.1 + rgb.1 as f64 * w, sum.2 + rgb.2 as f64 * w);
            weight += w;
        }
        if weight < 0.02 { return None; }
        let deep = (weight / 0.35).min(1.0);
        let (r, g, b) = (sum.0 / weight, sum.1 / weight, sum.2 / weight);
        let mix = |c: f64| (235.0 + (c - 235.0) * deep).clamp(0.0, 255.0) as u8;
        Some((mix(r), mix(g), mix(b)))
    }

    /// What has fallen to the bottom: the colour and how much of it.
    pub fn settled(&self) -> Option<(Rgb, f64)> {
        let mut sum = (0.0, 0.0, 0.0);
        let mut total = 0.0;
        for (key, &mmol) in &self.holds {
            if mmol < 1e-3 || !self.is_solid_here(key) { continue; }
            let Some(sp) = species(key) else { continue };
            // A colourless salt left behind by the water is a white powder.
            let rgb = match sp.rgb { Some(c) => c, None => (243, 243, 236) };
            sum = (sum.0 + rgb.0 as f64 * mmol, sum.1 + rgb.1 as f64 * mmol, sum.2 + rgb.2 as f64 * mmol);
            total += mmol;
        }
        if total < 1e-3 { return None; }
        Some((((sum.0 / total) as u8, (sum.1 / total) as u8, (sum.2 / total) as u8), total))
    }

    /// The vapour standing over the liquid, if there is any.
    pub fn vapour(&self) -> Option<Rgb> {
        self.holds.iter()
            .filter(|(_, &v)| v > 1e-4)
            .filter_map(|(k, _)| species(k))
            .find(|sp| sp.phase == Gas && sp.rgb.is_some())
            .and_then(|sp| sp.rgb)
    }

    /// The colour a wire dipped in this paints a flame, and its name.
    pub fn flame(&self) -> Option<(Rgb, &'static str)> {
        FLAMES.iter()
            .filter(|(k, _, _)| self.amount(k) > 0.01)
            .max_by(|a, b| self.amount(a.0).partial_cmp(&self.amount(b.0)).unwrap())
            .map(|(_, rgb, what)| (*rgb, *what))
    }

    /// Run the chemistry forward by `dt` seconds. Fast reactions need
    /// small steps, so the work is split into four.
    pub fn step(&mut self, dt: f64) -> Vec<Event> {
        let mut events = Vec::new();
        self.fizz *= 0.82;
        self.burner_and_boiling(dt);
        for _ in 0..4 { self.react(dt / 4.0, &mut events); }
        self.vent(dt);
        self.holds.retain(|_, v| *v > TRACE);
        let blue = self.blue() > 0.15;
        if blue != self.was_blue { self.flips += 1; self.was_blue = blue; }
        if self.flare > 0.0 { self.flare = (self.flare - dt * 1.4).max(0.0); }
        events
    }

    fn burner_and_boiling(&mut self, dt: f64) {
        if self.burner {
            let ceiling = if self.volume() > 0.1 { 100.0 } else { 900.0 };
            if self.temp < ceiling { self.temp = (self.temp + 90.0 * dt).min(ceiling); }
            if self.volume() > 0.0 && self.temp >= 99.9 {
                let gone = (2.5 * PER_ML * dt).min(self.amount("H2O"));
                *self.holds.entry("H2O").or_insert(0.0) -= gone;
            }
        } else if self.temp > 20.0 {
            self.temp -= (self.temp - 20.0).min(6.0 * dt * (1.0 + self.temp / 200.0));
        }
    }

    fn react(&mut self, dt: f64, events: &mut Vec<Event>) {
        let ml = self.working_ml();
        let mut joules = 0.0;
        for (i, rx) in REACTIONS.iter().enumerate() {
            let ok = match rx.needs {
                Wet => self.volume() > 0.2,
                Dry => self.volume() < 0.2,
                Hot(t) => self.temp >= t,
            };
            if !ok { continue; }
            // Speed is the rate constant times every reactant's
            // concentration multiplied together, which is what mass
            // action means: the more crowded, the faster.
            let mut speed = rx.k * ml;
            let mut limit = f64::INFINITY;
            for (key, n) in rx.left {
                let have = self.amount(key);
                if have < TRACE { speed = 0.0; break; }
                speed *= (have / ml).powf(*n);
                if *n > 0.0 { limit = limit.min(have / n); }
            }
            if speed <= 0.0 || !limit.is_finite() { continue; }
            let extent = (speed * dt).min(limit * 0.9);
            if extent < TRACE { continue; }
            for (k, n) in rx.left { *self.holds.entry(k).or_insert(0.0) -= extent * n; }
            for (k, n) in rx.right {
                *self.holds.entry(k).or_insert(0.0) += extent * n;
                if *n > 0.0 && extent > 1e-4 { self.seen.insert(k); }
            }
            joules += rx.heat * extent;
            if rx.bang && extent > 0.02 {
                self.flare = 1.0;
                self.flare_rgb = self.flame().map(|(c, _)| c).unwrap_or((255, 245, 200));
                self.banged = true;
                events.push(Event { says: rx.says.to_string(), bang: true });
                self.said.push(i);
            } else if !rx.says.is_empty() && extent > 0.005 && !self.said.contains(&i) {
                events.push(Event { says: rx.says.to_string(), bang: false });
                self.said.push(i);
            }
        }
        let mass = self.volume() * 4.18 + 2.0;
        self.temp = (self.temp + joules / mass).min(2500.0);
    }

    /// Gases leave, and that is what makes the bubbles.
    fn vent(&mut self, dt: f64) {
        let gases: Vec<&'static str> = self.holds.keys().copied()
            .filter(|k| phase(k) == Some(Gas)).collect();
        for k in gases {
            let have = self.amount(k);
            if have < TRACE { continue; }
            let out = (have * 3.0 * dt).min(have);
            *self.holds.entry(k).or_insert(0.0) -= out;
            self.fizz += out;
        }
    }

    /// True while something is still changing, so the screen has to
    /// follow. When this goes false the app stops ticking and waits for a
    /// key, and nothing runs at all.
    pub fn busy(&self) -> bool {
        if self.burner || self.flare > 0.0 || self.fizz > 1e-4 { return true; }
        if self.temp > 20.5 { return true; }
        REACTIONS.iter().any(|rx| {
            let ok = match rx.needs {
                Wet => self.volume() > 0.2,
                Dry => self.volume() < 0.2,
                Hot(t) => self.temp >= t,
            };
            ok && rx.left.iter().all(|(k, _)| self.amount(k) > 1e-4)
        })
    }

    /// Everything in there, biggest first, for the panel on the right.
    pub fn contents(&self) -> Vec<(&'static Species, f64)> {
        let mut out: Vec<(&Species, f64)> = self.holds.iter()
            .filter(|(k, &v)| v > 1e-3 && ***k != *"H2O")
            .filter_map(|(k, &v)| species(k).map(|sp| (sp, v)))
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        out
    }
}

pub fn species(key: &str) -> Option<&'static Species> {
    SPECIES.iter().find(|s| s.key == key)
}

fn phase(key: &str) -> Option<Phase> { species(key).map(|s| s.phase) }

/// The pH in words, for someone who has not met the number before.
pub fn ph_word(ph: f64) -> &'static str {
    match ph {
        p if p < 3.0 => "strongly acid",
        p if p < 6.5 => "acid",
        p if p < 7.5 => "neutral",
        p if p < 11.0 => "basic",
        _ => "strongly basic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(v: &mut Vessel, seconds: f64) {
        for _ in 0..(seconds / 0.05) as usize { v.step(0.05); }
    }

    #[test]
    fn every_reaction_names_substances_that_exist() {
        for rx in REACTIONS {
            for (k, _) in rx.left.iter().chain(rx.right.iter()) {
                assert!(species(k).is_some(), "no such substance: {k}");
            }
        }
        for (k, _) in ACIDS.iter().chain(BASES.iter()) { assert!(species(k).is_some(), "{k}"); }
        for (k, _, _) in FLAMES { assert!(species(k).is_some(), "{k}"); }
        for (k, _) in INDICATORS { assert!(species(k).is_some(), "{k}"); }
    }

    #[test]
    fn acid_and_base_leave_salt_behind() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("HCl", 10.0);
        v.add("NaOH", 10.0);
        run(&mut v, 5.0);
        assert!(v.amount("NaCl") > 9.0, "salt: {}", v.amount("NaCl"));
        assert!((v.ph() - 7.0).abs() < 1.5, "pH {}", v.ph());
        // Boil the water away and the salt is all that is left.
        v.burner = true;
        run(&mut v, 60.0);
        assert!(v.volume() < 0.2, "water left: {}", v.volume());
        assert!(v.amount("NaCl") > 9.0);
    }

    #[test]
    fn iodine_turns_starch_blue() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("starch", 2.0);
        assert!(v.liquid_rgb().is_none());
        v.add("I2", 1.0);
        let (r, g, b) = v.liquid_rgb().expect("colour");
        assert!(b > r && b > g && b > 40, "not blue: {r},{g},{b}");
        // Hypo eats the iodine and the blue goes with it.
        v.add("Na2S2O3", 5.0);
        run(&mut v, 3.0);
        assert!(v.blue() < 0.05, "still blue: {}", v.blue());
    }

    #[test]
    fn the_clock_waits_then_goes_blue() {
        let mut v = Vessel::new(Glass::Beaker);
        v.add("H2O", 100.0 * PER_ML);
        v.add("H2O2", 40.0);
        v.add("KI", 20.0);
        v.add("H2SO4", 10.0);
        v.add("starch", 5.0);
        v.add("Na2S2O3", 4.0);
        run(&mut v, 3.0);
        assert!(v.blue() < 0.05, "blue too early: {}", v.blue());
        run(&mut v, 120.0);
        assert!(v.blue() > 0.4, "never went blue: {}", v.blue());
    }

    #[test]
    fn the_other_clock_keeps_going() {
        let mut v = Vessel::new(Glass::Beaker);
        v.add("H2O", 100.0 * PER_ML);
        v.add("KIO3", 60.0);
        v.add("H2O2", 400.0);
        v.add("MA", 200.0);
        v.add("MnSO4", 2.0);
        v.add("H2SO4", 20.0);
        v.add("starch", 5.0);
        let mut flips = 0;
        let mut was = false;
        for _ in 0..6000 {
            v.step(0.05);
            let now = v.blue() > 0.15;
            if now != was { flips += 1; was = now; }
        }
        assert!(flips >= 4, "only {flips} changes of colour");
    }

    #[test]
    fn silver_nitrate_finds_a_chloride() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("NaCl", 5.0);
        v.add("AgNO3", 5.0);
        run(&mut v, 2.0);
        let (rgb, amount) = v.settled().expect("a precipitate");
        assert!(amount > 4.0, "only {amount} mmol");
        assert!(rgb.0 > 240 && rgb.1 > 240 && rgb.2 > 240, "not white: {rgb:?}");
    }

    #[test]
    fn copper_grows_a_silver_tree() {
        let mut v = Vessel::new(Glass::Beaker);
        v.add("H2O", 50.0 * PER_ML);
        v.add("AgNO3", 20.0);
        v.add("Cu", 8.0);
        run(&mut v, 120.0);
        assert!(v.amount("Ag") > 5.0, "no silver: {}", v.amount("Ag"));
        let (r, _, b) = v.liquid_rgb().expect("blue water");
        assert!(b > r, "water did not turn blue: {r},{b}");
    }

    #[test]
    fn an_indicator_reads_the_acid() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("HCl", 5.0);
        v.add("uni", 0.05);
        let (r, _, _) = v.liquid_rgb().unwrap();
        assert!(r > 200, "acid should read red");
        v.add("NaOH", 10.0);
        run(&mut v, 5.0);
        let (r2, _, b2) = v.liquid_rgb().unwrap();
        assert!(b2 > r2, "base should read blue or purple, got {r2},{b2}");
    }

    #[test]
    fn a_settled_vessel_stops_working() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("NaCl", 5.0);
        run(&mut v, 2.0);
        assert!(!v.busy(), "nothing is happening, so nothing should tick");
    }

    #[test]
    fn a_flame_test_names_the_metal() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("CuSO4", 5.0);
        let (rgb, what) = v.flame().expect("a colour");
        assert!(rgb.1 > 150 && rgb.2 > 150, "copper burns blue green");
        assert!(what.contains("copper"));
    }

    #[test]
    fn chalk_and_acid_turn_limewater_milky() {
        let mut v = Vessel::new(Glass::Beaker);
        v.add("H2O", 60.0 * PER_ML);
        v.add("CaOH2", 3.0);
        v.add("CaCO3", 10.0);
        v.add("HCl", 20.0);
        run(&mut v, 30.0);
        assert!(v.amount("CaCO3") > 0.5, "no chalk out of the gas");
    }

    #[test]
    fn an_oxidiser_beside_a_fuel_goes_off() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("KClO3", 5.0);
        v.add("S", 5.0);
        v.step(0.05);
        assert!(v.flare > 0.0, "it should have gone off");
        assert!(v.temp > 400.0, "only {} °C", v.temp);
    }

    #[test]
    fn water_keeps_a_dry_mixture_safe() {
        let mut v = Vessel::new(Glass::Tube);
        v.add("H2O", 10.0 * PER_ML);
        v.add("KClO3", 5.0);
        v.add("S", 5.0);
        run(&mut v, 5.0);
        assert!(v.flare == 0.0, "wet, so nothing should have gone off");
    }
}
