//! Comprehensive Edge Case Tests for Vietnamese IME Engine
//!
//! This test suite covers ALL scenarios a real user would encounter,
//! including edge cases that even experienced users might not think of.
//!
//! Run with: cargo test -p vietnamese-ime-engine --test comprehensive

use vnkey_core::{Engine, InputMode};

// ============================================================
// HELPER
// ============================================================
fn telex() -> Engine {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    cfg.modern_tone = true;
    e.set_config(cfg);
    e
}
fn vni() -> Engine {
    let mut e = Engine::new(InputMode::Vni);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    e
}

// ============================================================
// 1. BASIC TELEX TONES — Every single tone on every base vowel
// ============================================================

#[test]
fn tone_sac_all_vowels() {
    let cases = [
        ("as", "á"),
        ("es", "é"),
        ("is", "í"),
        ("os", "ó"),
        ("us", "ú"),
        ("ys", "ý"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

#[test]
fn tone_huyen_all_vowels() {
    let cases = [
        ("af", "à"),
        ("ef", "è"),
        ("if", "ì"),
        ("of", "ò"),
        ("uf", "ù"),
        ("yf", "ỳ"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

#[test]
fn tone_hoi_all_vowels() {
    let cases = [
        ("ar", "ả"),
        ("er", "ẻ"),
        ("ir", "ỉ"),
        ("or", "ỏ"),
        ("ur", "ủ"),
        ("yr", "ỷ"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

#[test]
fn tone_nga_all_vowels() {
    let cases = [
        ("ax", "ã"),
        ("ex", "ẽ"),
        ("ix", "ĩ"),
        ("ox", "õ"),
        ("ux", "ũ"),
        ("yx", "ỹ"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

#[test]
fn tone_nang_all_vowels() {
    let cases = [
        ("aj", "ạ"),
        ("ej", "ẹ"),
        ("ij", "ị"),
        ("oj", "ọ"),
        ("uj", "ụ"),
        ("yj", "ỵ"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

// ============================================================
// 2. MODIFIED VOWELS + TONES
// ============================================================

#[test]
fn tone_on_circumflex_vowels() {
    let cases = [
        ("aas", "ấ"),
        ("aaf", "ầ"),
        ("aar", "ẩ"),
        ("aax", "ẫ"),
        ("aaj", "ậ"),
        ("ees", "ế"),
        ("eef", "ề"),
        ("eer", "ể"),
        ("eex", "ễ"),
        ("eej", "ệ"),
        ("oos", "ố"),
        ("oof", "ồ"),
        ("oor", "ổ"),
        ("oox", "ỗ"),
        ("ooj", "ộ"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

#[test]
fn tone_on_breve_and_horn() {
    let cases = [
        ("aws", "ắ"),
        ("awf", "ằ"),
        ("awr", "ẳ"),
        ("awx", "ẵ"),
        ("awj", "ặ"),
        ("ows", "ớ"),
        ("owf", "ờ"),
        ("owr", "ở"),
        ("owx", "ỡ"),
        ("owj", "ợ"),
        ("uws", "ứ"),
        ("uwf", "ừ"),
        ("uwr", "ử"),
        ("uwx", "ữ"),
        ("uwj", "ự"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

// ============================================================
// 3. REAL VIETNAMESE WORDS — Common daily usage
// ============================================================

#[test]
fn common_words_telex() {
    let cases = [
        ("xin", "xin"),
        ("chaof", "chào"),
        ("camr", "cảm"),
        ("ddi", "đi"),
        ("hocj", "học"),
        ("vieejt", "việt"),
        ("nam", "nam"),
        ("thas", "thá"),
        ("bawns", "bắn"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

#[test]
fn complex_words_telex() {
    let cases = [("tieesng", "tiếng"), ("khoan", "khoan"), ("ddaats", "đất")];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed for input: {}", input);
    }
}

// ============================================================
// 4. QU SPECIAL CASES
// ============================================================

#[test]
fn qu_words() {
    let mut e = telex();
    // Progressive typing: q → qu → qua → quá
    assert_eq!(e.process_key('q'), "q");
    assert_eq!(e.process_key('u'), "qu");
    assert_eq!(e.process_key('a'), "qua");
    assert_eq!(e.process_key('s'), "quá"); // tone on 'a', NOT 'u'
    e.reset();

    assert_eq!(e.feed_str("quoocs"), "quốc"); // tone on 'ô'
}

// ============================================================
// 5. GI SPECIAL CASES
// ============================================================

#[test]
fn gi_words() {
    let mut e = telex();
    assert_eq!(e.process_key('g'), "g");
    assert_eq!(e.process_key('i'), "gi");
    assert_eq!(e.process_key('f'), "gì");
    e.reset();

    assert_eq!(e.feed_str("giaos"), "giáo"); // tone on 'a'
    e.reset();
    assert_eq!(e.feed_str("giuwax"), "giữa");
}

// ============================================================
// 6. DOUBLE-TAP CANCELLATION
// ============================================================

#[test]
fn double_tap_cancel() {
    let cases = [
        ("ass", "as"), // double s → cancel sắc
        ("aff", "af"), // double f → cancel huyền
        ("arr", "ar"), // double r → cancel hỏi
        ("axx", "ax"), // double x → cancel ngã
        ("ajj", "aj"), // double j → cancel nặng
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(
            e.feed_str(input),
            expected,
            "Failed for double-tap: {}",
            input
        );
    }
}

// ============================================================
// 7. Z REMOVES TONE
// ============================================================

#[test]
fn z_removes_tone() {
    let cases = [
        ("asz", "a"), // add sắc then remove
        ("afz", "a"), // add huyền then remove
        ("axz", "a"), // add ngã then remove
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(
            e.feed_str(input),
            expected,
            "Failed for z-remove: {}",
            input
        );
    }
}

#[test]
fn z_without_vowel_is_literal() {
    let mut e = telex();
    assert_eq!(e.process_key('z'), "z");
    e.reset();
    assert_eq!(e.feed_str("zz"), "zz");
}

// ============================================================
// 8. X WITHOUT VOWEL IS LITERAL
// ============================================================

#[test]
fn x_without_vowel_is_literal() {
    let mut e = telex();
    assert_eq!(e.process_key('x'), "x");
    e.reset();
    // "xa" = literal x + a (x as onset consonant)
    assert_eq!(e.feed_str("xa"), "xa");
}

// ============================================================
// 9. SMART W
// ============================================================

#[test]
fn smart_w() {
    let mut e = telex();
    // w at word start -> ư (Smart W)
    assert_eq!(e.process_key('w'), "ư");
    e.reset();
    // w after consonant → ư
    assert_eq!(e.feed_str("tw"), "tư");
    e.reset();
    // uw → ư (standard modifier)
    assert_eq!(e.feed_str("uw"), "ư");
    e.reset();
    // aw → ă (standard modifier)
    assert_eq!(e.feed_str("aw"), "ă");
    e.reset();
    // ow → ơ (standard modifier)
    assert_eq!(e.feed_str("ow"), "ơ");
}

// ============================================================
// 10. BRACKET KEYS (TELEX PRO)
// ============================================================

#[test]
fn bracket_keys() {
    let mut e = telex();
    assert_eq!(e.feed_str("h["), "hư");
    e.reset();
    assert_eq!(e.feed_str("h]"), "hơ");
}

// ============================================================
// 11. DD → Đ
// ============================================================

#[test]
fn dd_modifier() {
    let mut e = telex();
    assert_eq!(e.feed_str("dd"), "đ");
    e.reset();
    assert_eq!(e.feed_str("ddi"), "đi");
    e.reset();
    assert_eq!(e.feed_str("ddaats"), "đất");
}

// ============================================================
// 12. CASE PRESERVATION
// ============================================================

#[test]
fn uppercase_first_letter() {
    let mut e = telex();
    assert_eq!(e.feed_str("Vieejt"), "Việt");
    e.reset();
    assert_eq!(e.feed_str("Has"), "Há");
    e.reset();
    assert_eq!(e.feed_str("DDi"), "Đi");
}

// ============================================================
// 13. WORD BOUNDARY (Space & Punctuation)
// ============================================================

#[test]
fn word_boundary_space() {
    let mut e = telex();
    let r = e.feed_str("as ");
    assert_eq!(r, "á ");
    // Buffer should be reset after space
    assert_eq!(e.process_key('b'), "b");
}

#[test]
fn word_boundary_punctuation() {
    let mut e = telex();
    let r = e.feed_str("as.");
    assert_eq!(r, "á.");
    assert_eq!(e.process_key('b'), "b"); // new word
}

#[test]
fn word_boundary_comma() {
    let mut e = telex();
    let r = e.feed_str("as,");
    assert_eq!(r, "á,");
}

// ============================================================
// 14. DIPHTHONGS & TONE PLACEMENT
// ============================================================

#[test]
fn diphthong_tone_placement() {
    let cases = [
        // oa → tone on 'a' (second vowel)
        ("oans", "oán"),
        // oe → tone on 'e'
        ("oef", "oè"),
    ];
    for (input, expected) in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), expected, "Failed diphthong: {}", input);
    }
}

#[test]
fn triphthong_tone_placement() {
    let mut e = telex();
    // oai → tone on middle vowel 'a'
    assert_eq!(e.feed_str("oais"), "oái");
}

// ============================================================
// 15. DUPLICATE VOWELS (Bug from earlier)
// ============================================================

#[test]
fn duplicate_vowel_no_tone_jump() {
    // The key test: tone should NOT jump to wrong vowel
    // when user types duplicate vowels
    let mut e = telex();
    let result = e.feed_str("thayys");
    // Tone must be on 'a' — result must NOT have ý (tone on y)
    assert!(!result.contains('ý'), "Tone jumped to 'y'! Got: {}", result);
}

// ============================================================
// 16. VNI MODE — Complete coverage
// ============================================================

#[test]
fn vni_all_tones() {
    let cases = [
        ("a1", "á"),
        ("a2", "à"),
        ("a3", "ả"),
        ("a4", "ã"),
        ("a5", "ạ"),
    ];
    for (input, expected) in cases {
        let mut e = vni();
        assert_eq!(e.feed_str(input), expected, "VNI failed: {}", input);
    }
}

#[test]
fn vni_modifiers() {
    let cases = [
        ("a6", "â"),
        ("e6", "ê"),
        ("o6", "ô"),
        ("d9", "đ"),
        ("a8", "ă"),
        ("o7", "ơ"),
        ("u7", "ư"),
    ];
    for (input, expected) in cases {
        let mut e = vni();
        assert_eq!(
            e.feed_str(input),
            expected,
            "VNI modifier failed: {}",
            input
        );
    }
}

#[test]
fn vni_numbers_without_vowel_are_literal() {
    let mut e = vni();
    // '1' at start (no vowel) → literal
    assert_eq!(e.process_key('1'), "1");
    e.reset();
    assert_eq!(e.feed_str("b1"), "b1"); // no vowel before
}

// ============================================================
// 17. MODE SWITCHING
// ============================================================

#[test]
fn mode_switch_mid_session() {
    let mut e = telex();
    assert_eq!(e.feed_str("as"), "á"); // Telex
    e.set_mode(InputMode::Vni);
    assert_eq!(e.feed_str("a1"), "á"); // VNI
    e.set_mode(InputMode::Telex);
    assert_eq!(e.feed_str("af"), "à"); // Back to Telex
}

// ============================================================
// 18. CONSONANT-ONLY INPUT
// ============================================================

#[test]
fn consonant_only() {
    let cases = [
        "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "r", "t", "v", "th", "ch", "ng", "nh",
        "kh", "ph", "tr", "gi",
    ];
    for input in cases {
        let mut e = telex();
        assert_eq!(e.feed_str(input), input, "Consonant failed: {}", input);
    }
}

// ============================================================
// 19. SINGLE CHARACTER EDGE CASES
// ============================================================

#[test]
fn single_char_edge() {
    let mut e = telex();
    assert_eq!(e.process_key('a'), "a");
    e.reset();
    assert_eq!(e.process_key('s'), "s"); // s without vowel
    e.reset();
    assert_eq!(e.process_key('f'), "f");
    e.reset();
    assert_eq!(e.process_key('j'), "j");
    e.reset();
    assert_eq!(e.process_key('r'), "r");
    e.reset();
    assert_eq!(e.process_key('d'), "d");
}

// ============================================================
// 20. PROGRESSIVE TYPING (Key-by-key simulation)
// ============================================================

#[test]
fn progressive_typing_truong() {
    let mut e = telex();
    assert_eq!(e.process_key('t'), "t");
    assert_eq!(e.process_key('r'), "tr");
    assert_eq!(e.process_key('u'), "tru");
    assert_eq!(e.process_key('o'), "truo");
}

#[test]
fn progressive_typing_ban() {
    let mut e = telex();
    assert_eq!(e.process_key('b'), "b");
    assert_eq!(e.process_key('a'), "ba");
    assert_eq!(e.process_key('n'), "ban");
    assert_eq!(e.process_key('s'), "bán"); // tone on 'a'
}

#[test]
fn progressive_typing_viet() {
    let mut e = telex();
    assert_eq!(e.process_key('v'), "v");
    assert_eq!(e.process_key('i'), "vi");
    assert_eq!(e.process_key('e'), "vie");
    assert_eq!(e.process_key('e'), "viê"); // ee → ê
    assert_eq!(e.process_key('j'), "việ"); // nặng on ê
    assert_eq!(e.process_key('t'), "việt");
}

// ============================================================
// 21. RESET BEHAVIOR
// ============================================================

#[test]
fn reset_clears_state() {
    let mut e = telex();
    e.feed_str("as"); // "á"
    e.reset();
    // After reset, new word starts fresh
    assert_eq!(e.process_key('b'), "b");
    assert_eq!(e.process_key('a'), "ba");
    assert_eq!(e.process_key('f'), "bà");
}

// ============================================================
// 22. MULTIPLE TONES (Last one wins)
// ============================================================

#[test]
fn multiple_tones_last_wins() {
    let mut e = telex();
    // Type sắc, then huyền → huyền should win
    assert_eq!(e.feed_str("asf"), "à"); // s then f → last tone = huyền
}

// ============================================================
// 23. TONE AFTER CODA CONSONANT
// ============================================================

#[test]
fn tone_after_coda() {
    let mut e = telex();
    // "ans" → "án" (tone on 'a' even though 's' comes after 'n')
    assert_eq!(e.feed_str("ans"), "án");
    e.reset();
    assert_eq!(e.feed_str("angf"), "àng");
    e.reset();
    assert_eq!(e.feed_str("inhf"), "ình");
}

// ============================================================
// 24. EMPTY & MINIMAL EDGE CASES
// ============================================================

#[test]
fn empty_feed_str() {
    let mut e = telex();
    // Empty string should return empty
    let r = e.feed_str("");
    assert_eq!(r, "");
}

#[test]
fn only_space() {
    let mut e = telex();
    assert_eq!(e.process_key(' '), " ");
}

// ============================================================
// 25. NUMBERS IN TELEX MODE
// ============================================================

#[test]
fn numbers_pass_through_telex() {
    let mut e = telex();
    assert_eq!(e.process_key('1'), "1");
    e.reset();
    assert_eq!(e.process_key('0'), "0");
}

// ============================================================
// 26. RAPID CONSECUTIVE WORDS
// ============================================================

#[test]
fn two_words_with_space() {
    let mut e = telex();
    // "toi di" → "tôi đi" ... let's test simpler
    let r1 = e.feed_str("as ");
    assert_eq!(r1, "á ");
    let r2 = e.feed_str("bf");
    // After space reset, this is a new word
    assert_eq!(r2, "bf"); // b has no vowel, f is literal
}
