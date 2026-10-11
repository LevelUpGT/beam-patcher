# BEAM Patcher — Windows GUI Preview

**Bu paket bir grafik penceresi açar.** Önceki \`thor_smoke.exe\` gibi konsol aracı değildir.

- Dosya: \`BEAM-Patcher-Preview.exe\`
- İşletim sistemi: Windows 10/11 x64 (Microsoft Edge WebView2 Runtime gerekli)
- Güvenli önizleme: oyun yükleme, güncelleme, silme, GRF birleştirme, otomatik updater ve sunucu bağlantıları **kapalı**
- İşlev: \`THOR Dosyası Seç ve Test Et\` ile yerel **loose-client THOR** paketini BEAM'in Rust doğrulayıcısı kullanarak yalnızca kendi oluşturduğu geçici klasörde uygular, sonuç dosyalarını karşılaştırır, geçici klasörü temizler. Gerçek Ragnarok dizini istenmez.
- \`Sürüm Bilgisi\` backend komutunu çağırır. \`Kapat\` pencereyi kapatır.
- Legacy 2015 GRF-merging THOR arşivleri bu testten *bilerek* geçmez; eski 2015 Thor Patcher aynen korunur.

## Çalıştırma

1. GitHub Actions \`Windows GUI Preview\` testinin PASS olduğunu doğrula.
2. İlgili çalışmanın Artifacts bölümünden \`BEAM-Patcher-Windows-GUI-Preview\` ZIP'ini indir.
3. ZIP'i aç; \`BEAM-Patcher-Preview.exe\` üzerine çift tıkla. *Ragnarok oyun klasörüne kopyalama.*
4. \`THOR Dosyası Seç ve Test Et\` düğmesine basarak \`turkuaz-2022-hybrid-8cfc3211f8eb.thor\` gibi **yerel** bir THOR seç.
5. Test dosyaları geçici dizinde işlendiğinde sonuç uygulamada gösterilir. Gerçek istemci dosyaları değişmez.

Başlatmada pencere gelmezse \`Microsoft Edge WebView2 Runtime\` kurulumu / antivirüs uyarısı ve Windows Olay Görüntüleyicisi hata kaydını kontrol et. Windows CI derlemesi **tek başına** kullanıcı bilgisayarındaki açılışı doğrulamaz; kullanıcının ekran görüntüsü ile doğrulama gerekir.

## Önceki konsol aracıyla fark

\`thor_smoke.exe\` yalnızca komut satırı test aracıdır ve çift tıkla pencere açmaz. Bu proje gerçek bir **Tauri Windows desktop GUI** oluşturur. Şimdilik önizlemede OYNA/Güncelle özellikleri kapalıdır; gerçek istemciye bağlanacak tam launcher ilerideki ayrı doğrulama aşamasıdır.

Kaynak: \`beam-ui/preview-web/index.html\`, \`beam-ui/src/preview.rs\`, \`beam-ui/tauri.preview.conf.json\`, \`beam-patcher/src/preview.rs\`. İstemciler: 2015 Thor, yeni 2015 rPatchur, 2022 rPatchur kanalları mevcut halleriyle çalışmaya devam eder.
