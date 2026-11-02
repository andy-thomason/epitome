//! Reserved keywords and the `` `begin_keywords `` versions that reserve
//! them (Annex B, 22.14).

/// A version of the language, as named by `` `begin_keywords ``.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Version {
    V1364_1995,
    V1364_2001,
    V1364_2005,
    V1800_2005,
    V1800_2009,
    V1800_2012,
    V1800_2017,
    V1800_2023,
}

/// A set of reserved keywords: those of a version, and for
/// `"1364-2001-noconfig"` that version without the configuration words.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Keywords {
    pub version: Version,
    pub noconfig: bool,
}

impl Default for Keywords {
    fn default() -> Self {
        Keywords::LATEST
    }
}

impl Keywords {
    /// The set used when no `` `begin_keywords `` is in effect.
    pub const LATEST: Keywords = Keywords { version: Version::V1800_2023, noconfig: false };

    /// The set named by a `` `begin_keywords `` version specifier, without
    /// its quotes.
    pub fn from_specifier(s: &str) -> Option<Keywords> {
        use Version::*;
        let (version, noconfig) = match s {
            "1800-2023" => (V1800_2023, false),
            "1800-2017" => (V1800_2017, false),
            "1800-2012" => (V1800_2012, false),
            "1800-2009" => (V1800_2009, false),
            "1800-2005" => (V1800_2005, false),
            "1364-2005" => (V1364_2005, false),
            "1364-2001" => (V1364_2001, false),
            "1364-2001-noconfig" => (V1364_2001, true),
            "1364-1995" => (V1364_1995, false),
            _ => return None,
        };
        Some(Keywords { version, noconfig })
    }

    /// Whether `word` is reserved in this set.
    pub fn contains(self, word: &str) -> bool {
        match introduced(word) {
            Some(v) => v <= self.version && !(self.noconfig && CONFIG_WORDS.contains(&word)),
            None => false,
        }
    }
}

/// The version that first reserved `word`, if any version does.
pub fn introduced(word: &str) -> Option<Version> {
    KEYWORDS.binary_search_by(|(w, _)| w.cmp(&word)).ok().map(|i| KEYWORDS[i].1)
}

/// The words of 1364-2001 that `"1364-2001-noconfig"` leaves out.
const CONFIG_WORDS: &[&str] =
    &["cell", "config", "design", "endconfig", "incdir", "include", "instance", "liblist", "library", "use"];

use Version::*;

/// Every reserved word, sorted, with the version that first reserved it.
/// 1800-2017 and 1800-2023 reserve no new words.
pub const KEYWORDS: &[(&str, Version)] = &[
    ("accept_on", V1800_2009),
    ("alias", V1800_2005),
    ("always", V1364_1995),
    ("always_comb", V1800_2005),
    ("always_ff", V1800_2005),
    ("always_latch", V1800_2005),
    ("and", V1364_1995),
    ("assert", V1800_2005),
    ("assign", V1364_1995),
    ("assume", V1800_2005),
    ("automatic", V1364_2001),
    ("before", V1800_2005),
    ("begin", V1364_1995),
    ("bind", V1800_2005),
    ("bins", V1800_2005),
    ("binsof", V1800_2005),
    ("bit", V1800_2005),
    ("break", V1800_2005),
    ("buf", V1364_1995),
    ("bufif0", V1364_1995),
    ("bufif1", V1364_1995),
    ("byte", V1800_2005),
    ("case", V1364_1995),
    ("casex", V1364_1995),
    ("casez", V1364_1995),
    ("cell", V1364_2001),
    ("chandle", V1800_2005),
    ("checker", V1800_2009),
    ("class", V1800_2005),
    ("clocking", V1800_2005),
    ("cmos", V1364_1995),
    ("config", V1364_2001),
    ("const", V1800_2005),
    ("constraint", V1800_2005),
    ("context", V1800_2005),
    ("continue", V1800_2005),
    ("cover", V1800_2005),
    ("covergroup", V1800_2005),
    ("coverpoint", V1800_2005),
    ("cross", V1800_2005),
    ("deassign", V1364_1995),
    ("default", V1364_1995),
    ("defparam", V1364_1995),
    ("design", V1364_2001),
    ("disable", V1364_1995),
    ("dist", V1800_2005),
    ("do", V1800_2005),
    ("edge", V1364_1995),
    ("else", V1364_1995),
    ("end", V1364_1995),
    ("endcase", V1364_1995),
    ("endchecker", V1800_2009),
    ("endclass", V1800_2005),
    ("endclocking", V1800_2005),
    ("endconfig", V1364_2001),
    ("endfunction", V1364_1995),
    ("endgenerate", V1364_2001),
    ("endgroup", V1800_2005),
    ("endinterface", V1800_2005),
    ("endmodule", V1364_1995),
    ("endpackage", V1800_2005),
    ("endprimitive", V1364_1995),
    ("endprogram", V1800_2005),
    ("endproperty", V1800_2005),
    ("endsequence", V1800_2005),
    ("endspecify", V1364_1995),
    ("endtable", V1364_1995),
    ("endtask", V1364_1995),
    ("enum", V1800_2005),
    ("event", V1364_1995),
    ("eventually", V1800_2009),
    ("expect", V1800_2005),
    ("export", V1800_2005),
    ("extends", V1800_2005),
    ("extern", V1800_2005),
    ("final", V1800_2005),
    ("first_match", V1800_2005),
    ("for", V1364_1995),
    ("force", V1364_1995),
    ("foreach", V1800_2005),
    ("forever", V1364_1995),
    ("fork", V1364_1995),
    ("forkjoin", V1800_2005),
    ("function", V1364_1995),
    ("generate", V1364_2001),
    ("genvar", V1364_2001),
    ("global", V1800_2009),
    ("highz0", V1364_1995),
    ("highz1", V1364_1995),
    ("if", V1364_1995),
    ("iff", V1800_2005),
    ("ifnone", V1364_1995),
    ("ignore_bins", V1800_2005),
    ("illegal_bins", V1800_2005),
    ("implements", V1800_2012),
    ("implies", V1800_2009),
    ("import", V1800_2005),
    ("incdir", V1364_2001),
    ("include", V1364_2001),
    ("initial", V1364_1995),
    ("inout", V1364_1995),
    ("input", V1364_1995),
    ("inside", V1800_2005),
    ("instance", V1364_2001),
    ("int", V1800_2005),
    ("integer", V1364_1995),
    ("interconnect", V1800_2012),
    ("interface", V1800_2005),
    ("intersect", V1800_2005),
    ("join", V1364_1995),
    ("join_any", V1800_2005),
    ("join_none", V1800_2005),
    ("large", V1364_1995),
    ("let", V1800_2009),
    ("liblist", V1364_2001),
    ("library", V1364_2001),
    ("local", V1800_2005),
    ("localparam", V1364_2001),
    ("logic", V1800_2005),
    ("longint", V1800_2005),
    ("macromodule", V1364_1995),
    ("matches", V1800_2005),
    ("medium", V1364_1995),
    ("modport", V1800_2005),
    ("module", V1364_1995),
    ("nand", V1364_1995),
    ("negedge", V1364_1995),
    ("nettype", V1800_2012),
    ("new", V1800_2005),
    ("nexttime", V1800_2009),
    ("nmos", V1364_1995),
    ("nor", V1364_1995),
    ("noshowcancelled", V1364_2001),
    ("not", V1364_1995),
    ("notif0", V1364_1995),
    ("notif1", V1364_1995),
    ("null", V1800_2005),
    ("or", V1364_1995),
    ("output", V1364_1995),
    ("package", V1800_2005),
    ("packed", V1800_2005),
    ("parameter", V1364_1995),
    ("pmos", V1364_1995),
    ("posedge", V1364_1995),
    ("primitive", V1364_1995),
    ("priority", V1800_2005),
    ("program", V1800_2005),
    ("property", V1800_2005),
    ("protected", V1800_2005),
    ("pull0", V1364_1995),
    ("pull1", V1364_1995),
    ("pulldown", V1364_1995),
    ("pullup", V1364_1995),
    ("pulsestyle_ondetect", V1364_2001),
    ("pulsestyle_onevent", V1364_2001),
    ("pure", V1800_2005),
    ("rand", V1800_2005),
    ("randc", V1800_2005),
    ("randcase", V1800_2005),
    ("randsequence", V1800_2005),
    ("rcmos", V1364_1995),
    ("real", V1364_1995),
    ("realtime", V1364_1995),
    ("ref", V1800_2005),
    ("reg", V1364_1995),
    ("reject_on", V1800_2009),
    ("release", V1364_1995),
    ("repeat", V1364_1995),
    ("restrict", V1800_2009),
    ("return", V1800_2005),
    ("rnmos", V1364_1995),
    ("rpmos", V1364_1995),
    ("rtran", V1364_1995),
    ("rtranif0", V1364_1995),
    ("rtranif1", V1364_1995),
    ("s_always", V1800_2009),
    ("s_eventually", V1800_2009),
    ("s_nexttime", V1800_2009),
    ("s_until", V1800_2009),
    ("s_until_with", V1800_2009),
    ("scalared", V1364_1995),
    ("sequence", V1800_2005),
    ("shortint", V1800_2005),
    ("shortreal", V1800_2005),
    ("showcancelled", V1364_2001),
    ("signed", V1364_2001),
    ("small", V1364_1995),
    ("soft", V1800_2012),
    ("solve", V1800_2005),
    ("specify", V1364_1995),
    ("specparam", V1364_1995),
    ("static", V1800_2005),
    ("string", V1800_2005),
    ("strong", V1800_2009),
    ("strong0", V1364_1995),
    ("strong1", V1364_1995),
    ("struct", V1800_2005),
    ("super", V1800_2005),
    ("supply0", V1364_1995),
    ("supply1", V1364_1995),
    ("sync_accept_on", V1800_2009),
    ("sync_reject_on", V1800_2009),
    ("table", V1364_1995),
    ("tagged", V1800_2005),
    ("task", V1364_1995),
    ("this", V1800_2005),
    ("throughout", V1800_2005),
    ("time", V1364_1995),
    ("timeprecision", V1800_2005),
    ("timeunit", V1800_2005),
    ("tran", V1364_1995),
    ("tranif0", V1364_1995),
    ("tranif1", V1364_1995),
    ("tri", V1364_1995),
    ("tri0", V1364_1995),
    ("tri1", V1364_1995),
    ("triand", V1364_1995),
    ("trior", V1364_1995),
    ("trireg", V1364_1995),
    ("type", V1800_2005),
    ("typedef", V1800_2005),
    ("union", V1800_2005),
    ("unique", V1800_2005),
    ("unique0", V1800_2009),
    ("unsigned", V1364_2001),
    ("until", V1800_2009),
    ("until_with", V1800_2009),
    ("untyped", V1800_2009),
    ("use", V1364_2001),
    ("uwire", V1364_2005),
    ("var", V1800_2005),
    ("vectored", V1364_1995),
    ("virtual", V1800_2005),
    ("void", V1800_2005),
    ("wait", V1364_1995),
    ("wait_order", V1800_2005),
    ("wand", V1364_1995),
    ("weak", V1800_2009),
    ("weak0", V1364_1995),
    ("weak1", V1364_1995),
    ("while", V1364_1995),
    ("wildcard", V1800_2005),
    ("wire", V1364_1995),
    ("with", V1800_2005),
    ("within", V1800_2005),
    ("wor", V1364_1995),
    ("xnor", V1364_1995),
    ("xor", V1364_1995),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted() {
        assert!(KEYWORDS.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(KEYWORDS.len(), 248);
    }

    #[test]
    fn versions() {
        let k = |s| Keywords::from_specifier(s).unwrap();
        assert!(k("1364-1995").contains("module") && !k("1364-1995").contains("generate"));
        assert!(k("1364-2001").contains("generate") && k("1364-2001").contains("config"));
        assert!(!k("1364-2001-noconfig").contains("config") && k("1364-2001-noconfig").contains("generate"));
        assert!(!k("1364-2005").contains("logic") && k("1364-2005").contains("uwire"));
        assert!(k("1800-2005").contains("logic") && !k("1800-2005").contains("checker"));
        assert!(k("1800-2009").contains("checker") && !k("1800-2009").contains("soft"));
        assert!(k("1800-2017").contains("soft") && Keywords::LATEST.contains("nettype"));
        assert!(Keywords::from_specifier("1800-2024").is_none());
    }
}
