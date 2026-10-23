<!--
Dosya Yolu: docs/LEVELUP-GAMES-BEAM-QA.md
Amac: Sac Modelleri ve LevelUp Games videosu icin Beam test akisinin sinirlarini tanimlar
Modul - Documentation Markdown
Version: 0.1.0
Aciklama: Kaynak kodundan yerel medya uretme ve offline Beam paketlerini test etme
Bagimli Oldugu Katman: Beam UI | RagnarokClient PR #17
-->

# LevelUp Games / Sac Modelleri - Beam QA

Bu dokuman **QA** amaclidir, yayinlama veya mevcut oyuncu dosyalarini
guncelleme talimati degildir. Client veya patch feed varsayilani degismez.

## 1. LevelUp Games videosunu Git kaynak kodundan uret

Bu repoda tools/render_levelup_games_intro.py bulunur.
Sistemde FFmpeg / libx264 / aac / drawtext kuruluysa:

    python ./tools/render_levelup_games_intro.py

Cikti: beam-ui/public/assets/levelup-games-intro.mp4.

MP4 dosyasi indirilmeksizin yeni bir QA videosu kaynak kodundan uretilir.
Onceden mevcut dosyanin ustune yazilmaz. Uretilen dosya, RagnarokClient'da
sabit SHA256 ile kayitli **orijinal** LevelUp Games videosundan farkli
bir yerel QA renderidir. Bu surumle EXE'ye video gomulmez.

Mevcut Beam config.yml app bolumundeki ilgili alanlar:

    app:
      video_background_enabled: true
      video_background_file: "assets/levelup-games-intro.mp4"

Yukardaki alanlar mevcut app alanina birlestirilir; tek basina gecerli
tam config degildir. Beam medya yuklemesi ve istemci davranisi gercek
Windows sisteminde ayrica smoke edilmelidir. UI background videosu,
Windows EXE splash animasyonuyla ayni sey degildir.

## 2. Sac Modelleri BEAM QA paketini olustur

Asset kaynagi RagnarokClient PR #17 dalidir. Daha once sahipte mevcut olan
orijinal kaynak ZIP ile asagidaki islemleri RagnarokClient repo kokunde yap:

    python ./tools/STAGE-HAIR-PACK.py /path/to/titanclient-Modifications-master.zip --stage
    python ./tools/BUILD-BEAM-HAIR-PACK.py

Sonuc: patches/output/hair-pack-84.beam (BEAM v1, 336 ACT/SPR, UTF-8
isimler). Program input QA manifest, dosya SHA256, tam kapsam,
duplicate, BEAM v1 MD5 ve readback dogrulamasini yapar. Mevcut
patchlist'e asla eklemez ve GRF/EXE dosyalarini degistirmez.

**KRITIK UYUMLULUK:** Ragnarok 20220406 client Korece GRF dosya adlari
icin CP949 bekleyebilir. Mevcut Beam GRF yazici filename.as_bytes()
ile UTF-8 dosya adi yazar. CP949 oyun gorunurlugu **dogrulanmadi**.
Ayrica mevcut hairstyle.grf ile isim cakismalari bulunuyor.
hair-pack-84.beam dosyasini aktif ragnardata.grf/tdata1.grf
uzerine uygulama; yalnizca izole kopya ve rollback testi uygundur.

## 3. Public 2022 Beam dagitimi kapali kalir

b1glord/ragnarokclientpatch/2022/beam/channel-policy.json:
ready=false, publish_allowed=false, published_patch_count=0.
Bu QA akisi policy, patchlist, client EXE, DATA.INI veya GRF degistirmez.

Kaynak PR'ler:
- https://github.com/LevelUpGT/RagnarokClient/pull/17
- https://github.com/LevelUpGT/RagnarokServer/pull/241

Ucuncu taraf sac sprite'larinin yeniden dagitim izni netlesmeden
binary assetlerin public repoya eklenmesi ertelenmistir.
