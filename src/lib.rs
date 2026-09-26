//! Türkçe katlama — araçların arama öncesi metni aynı biçime getirdiği TEK yer.
//!
//! Neden ortak: el-Fihrist (`belirtecle`) ve pasli-beyin (`katla`/`tokenler`)
//! ayrı ayrı yazmıştı ve ayrışmıştı. Biri "kaynak—bak"ı tek, öbürü iki belirteç
//! sayıyordu; ikisi de şapkayı katlamıyordu ("kâtip" ≠ "katip"), ayrışık
//! yazılmış harf birinde sözcüğü ikiye bölüyordu. İki katlama arasındaki fark,
//! iki aracın aynı sözcüğü farklı bulması demektir.
//!
//! Kural (altin-kapi ADR 0002): ayrıştır (NFD) → birleşik işaretleri at →
//! dile duyarlı küçük harf. Sonuç: İ/I/ı/i → i, â→a, ş→s, ḥ→h, é→e; ayrışık ve
//! birleşik yazım aynı sonucu verir. Rust'ın `to_lowercase`'i dile duyarsızdır
//! ('İ' → "i̇"), o yüzden İ/I önce elle çevrilir.
//!
//! ponytail: aramaya yönelik, geri dönüşsüz. Görüntülenecek metin için DEĞİL.

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

/// Metni arama biçimine katlar: küçük harf, aksansız, Türkçe İ/ı birleşik.
pub fn katla(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for c in metin.nfd() {
        if is_combining_mark(c) {
            continue;
        }
        match c {
            // NFD 'İ'yi 'I' + U+0307'ye ayırır; nokta yukarıda atıldı.
            'I' | 'ı' => cikti.push('i'),
            _ => cikti.extend(c.to_lowercase()),
        }
    }
    cikti
}

/// Katlanmış metni belirteçlere böler. Harf/rakam dışındaki her şey ayraçtır
/// (ASCII dışı noktalama dahil: `—`, `’`, `“`).
pub fn belirtecle(metin: &str) -> Vec<String> {
    katla(metin)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn turkce_harfler() {
        assert_eq!(katla("İSTANBUL ışık ÇĞÖŞÜ"), "istanbul isik cgosu");
        assert_eq!(katla("IŞIK"), "isik");
    }

    #[test]
    fn sapka_ve_transliterasyon() {
        assert_eq!(katla("Kitâbü'l-Fihrist"), "kitabu'l-fihrist");
        assert_eq!(katla("Muḥammad ṣaḥīḥ"), "muhammad sahih");
        assert_eq!(katla("İbnü'n-Nedîm"), "ibnu'n-nedim");
    }

    #[test]
    fn ayrisik_ve_birlesik_ayni() {
        let birlesik = "k\u{e2}tip";
        let ayrisik = "ka\u{302}tip";
        assert_eq!(katla(birlesik), katla(ayrisik));
        assert_eq!(belirtecle(ayrisik), vec!["katip"]);
    }

    #[test]
    fn ascii_disi_noktalama_ayractir() {
        assert_eq!(
            belirtecle("kaynak—bak “alıntı”"),
            vec!["kaynak", "bak", "alinti"]
        );
    }

    #[test]
    fn arapca_harf_korunur() {
        // Arapça harfler aksan değil, harftir; yalnız hareke (birleşik) atılır.
        assert_eq!(belirtecle("كِتَاب"), vec!["كتاب"]);
    }
}
