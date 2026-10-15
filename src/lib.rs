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

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// U+0307 (üstte nokta). Çıplak `to_lowercase` 'İ'den "i" + bunu üretir; böyle
/// bozulmuş metinde i'nin ardındaki nokta fazladır, atılır.
const NOKTA: char = '\u{307}';

/// Metni arama biçimine katlayıp verilen tampona yazar: küçük harf, aksansız, Türkçe İ/ı birleşik.
/// Tamponu temizler (`clear`) ve gerekiyorsa kapasite ayırır.
#[inline]
pub fn katla_tampona(metin: &str, cikti: &mut String) {
    cikti.clear();
    cikti.reserve(metin.len());
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
}

/// Metni arama biçimine katlar: küçük harf, aksansız, Türkçe İ/ı birleşik.
#[inline]
pub fn katla(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    katla_tampona(metin, &mut cikti);
    cikti
}

/// Görüntülenecek/saklanacak metin için küçük harf: işaretler KORUNUR (ş, ç, â),
/// yalnız büyük/küçük harf birleşir. Arama için değil, onun için `katla`.
///
/// `to_lowercase` 'İ'den "i̇" (i + U+0307) üretir; burada 'İ' → 'i'. Dil
/// bilinmediği için 'I' de 'i' olur (İngilizce kısaltmalar: "API" → "api");
/// 'ı' olduğu gibi kalır. NFC'ye getirir: ayrışık ve birleşik yazım aynı çıkar.
/// Yasa: `katla(x) == katla(&kucult(x))` — arama katlaması bunun üstündedir.
pub fn kucult(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for c in metin.nfc() {
        match c {
            'İ' | 'I' => cikti.push('i'),
            NOKTA if cikti.ends_with('i') => {}
            _ => cikti.extend(c.to_lowercase()),
        }
    }
    cikti
}

/// Türkçe yazım kuralıyla küçük harf: 'I' → 'ı', 'İ' → 'i'; işaret korunur.
/// Metnin Türkçe olduğu BİLİNİYORSA (künye, tez metni). Dil bilinmiyorsa
/// `kucult` ("API" → "apı" olmasın). NFC'ye getirir.
pub fn tr_kucuk(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for c in metin.nfc() {
        match c {
            'İ' => cikti.push('i'),
            'I' => cikti.push('ı'),
            NOKTA if cikti.ends_with('i') => {}
            _ => cikti.extend(c.to_lowercase()),
        }
    }
    cikti
}

/// Türkçe yazım kuralıyla büyük harf: 'i' → 'İ', 'ı' → 'I'. `to_uppercase`
/// "çiçek"i "ÇIÇEK" yapar; bu "ÇİÇEK". NFC'ye getirir.
pub fn tr_buyuk(metin: &str) -> String {
    let mut cikti = String::with_capacity(metin.len());
    for c in metin.nfc() {
        match c {
            'i' => cikti.push('İ'),
            'ı' => cikti.push('I'),
            NOKTA if cikti.ends_with('İ') => {}
            _ => cikti.extend(c.to_uppercase()),
        }
    }
    cikti
}

/// Yalnız ilk harfi Türkçe kuralla büyütür, gerisine dokunmaz ("istanbul" →
/// "İstanbul"). Başlık düzeni kuran kod her sözcüğe bunu uygular.
pub fn tr_bas_buyuk(kelime: &str) -> String {
    let mut harfler = kelime.chars();
    match harfler.next() {
        Some(ilk) => {
            let mut cikti = String::with_capacity(kelime.len() + 2);
            match ilk {
                'i' => cikti.push('İ'),
                'ı' => cikti.push('I'),
                _ => cikti.extend(ilk.to_uppercase()),
            }
            cikti.push_str(harfler.as_str());
            cikti
        }
        None => String::new(),
    }
}

/// Sıfır-tahsisli belirteç yineleyicisi. Harf/rakam dışındaki her karakteri ayraç sayar.
#[inline]
pub fn belirtecler(metin: &str) -> impl Iterator<Item = &str> {
    metin
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
}

/// Katlanmış metni belirteçlere böler. Harf/rakam dışındaki her şey ayraçtır
/// (ASCII dışı noktalama dahil: `—`, `’`, `“`).
pub fn belirtecle(metin: &str) -> Vec<String> {
    let katlanmis = katla(metin);
    belirtecler(&katlanmis).map(str::to_string).collect()
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
    fn kucult_isareti_korur_i_artigi_birakmaz() {
        assert_eq!(kucult("İSTANBUL Şehir"), "istanbul şehir");
        assert_eq!(kucult("API ılık"), "api ılık");
        // Ayrışık yazılmış ş birleşik çıkar.
        assert_eq!(kucult("s\u{327}ehir"), "şehir");
        assert!(!kucult("İ").contains('\u{307}'));
    }

    #[test]
    fn turkce_yazim_kurali() {
        assert_eq!(tr_kucuk("IŞIK İSTANBUL Çiçek"), "ışık istanbul çiçek");
        assert_eq!(tr_buyuk("çiçek ılık"), "ÇİÇEK ILIK");
        // zopay pdf_tara: to_lowercase "AÇIKÖĞRETİM"i "açiköğreti̇m" yapıp künyeyi kaçırıyordu.
        assert_eq!(tr_kucuk("AÇIKÖĞRETİM HAZIRLAMA"), "açıköğretim hazırlama");
        assert_eq!(tr_bas_buyuk("istanbul"), "İstanbul");
        assert_eq!(tr_bas_buyuk("ırmak"), "Irmak");
        assert_eq!(tr_bas_buyuk(""), "");
        // Ayrışık yazılmış İ (I + U+0307) de 'i' olur, "ı̇" değil.
        assert_eq!(tr_kucuk("I\u{307}stanbul"), "istanbul");
        assert_eq!(tr_kucuk(&tr_buyuk("çiçekçi ılık")), "çiçekçi ılık");
    }

    #[test]
    fn to_lowercase_artigi_onarilir() {
        // "İstanbul".to_lowercase() == "i\u{307}stanbul": nokta i'de kalmamalı.
        let bozuk = "İstanbul".to_lowercase();
        assert_eq!(kucult(&bozuk), "istanbul");
        assert_eq!(tr_kucuk(&bozuk), "istanbul");
        assert_eq!(tr_buyuk(&bozuk), "İSTANBUL");
        assert_eq!(katla(&bozuk), "istanbul");
        // Yasa bozuk girdide de tutar.
        assert_eq!(katla(&bozuk), katla(&kucult(&bozuk)));
    }

    #[test]
    fn katla_kucultun_ustundedir() {
        for x in [
            "İSTANBUL ışık ÇĞÖŞÜ",
            "Kitâbü'l-Fihrist",
            "IŞIK",
            "ka\u{302}tip",
            "كِتَاب",
        ] {
            assert_eq!(katla(x), katla(&kucult(x)), "{x}");
            assert_eq!(katla(x), katla(&tr_kucuk(x)), "{x}");
        }
    }

    #[test]
    fn arapca_harf_korunur() {
        // Arapça harfler aksan değil, harftir; yalnız hareke (birleşik) atılır.
        assert_eq!(belirtecle("كِتَاب"), vec!["كتاب"]);
    }

    #[test]
    fn katla_tampona_ve_belirtecler_sifir_tahsis_dogru() {
        let mut buf = String::new();
        katla_tampona("İstanbul Işık", &mut buf);
        assert_eq!(buf, "istanbul isik");
        let tokens: Vec<&str> = belirtecler(&buf).collect();
        assert_eq!(tokens, vec!["istanbul", "isik"]);
    }
}

#[cfg(test)]
mod proptest_testleri {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn katla_idempotenttir(s in "\\PC*") {
            let k1 = katla(&s);
            let k2 = katla(&k1);
            prop_assert_eq!(k1, k2);
        }

        #[test]
        fn katla_tampona_esittir_katla(s in "\\PC*") {
            let mut buf = String::new();
            katla_tampona(&s, &mut buf);
            prop_assert_eq!(katla(&s), buf);
        }
    }
}
