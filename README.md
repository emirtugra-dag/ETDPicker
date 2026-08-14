# 🎨 ETDPicker

<div align="center">
  <img src="assets/app_icon.png" width="128" height="128" alt="ETDPicker Logo" />
  <h3>Ultra Hafif, Hızlı ve Hassas Ekran Renk Seçici</h3>
  <p>Windows için sıfır gecikmeli, ultra düşük RAM tüketen (~2-4 MB), çoklu dil destekli modern renk seçme aracı.</p>
</div>

---

## 🌟 Özellikler (Features)

- 🚀 **Ultra Düşük RAM ve CPU Kullanımı**: Saf Rust ve doğrudan yerel Windows API ile geliştirildi. Boşta %0 CPU, yalnızca ~2-4 MB RAM harcar.
- 🔍 **Piksel Büyüteci (Loupe / Magnifier)**: İmlecin etrafındaki alanı 10x yakınlaştırarak ızgara (grid) üzerinde piksel hassasiyetinde renk seçimi sağlar.
- ⌨️ **Özelleştirilebilir Global Kısayol Tuşu**: Varsayılan olarak `Alt + P` kısayolu ile sistemin her yerinden anında çalışır. Ayarlar menüsünden dilediğiniz tuş kombinasyonuna değiştirilebilir.
- 🎨 **Zengin Renk Formatları & Tek Tıkla Kopyalama**:
  - **HEX**: `#FF5733`
  - **RGB (Paint Formatı)**: `255, 87, 51`
  - **Bileşenler**: Kırmızı (R), Yeşil (G), Mavi (B) değerleri
  - **HSL, HSV, CMYK**
- 🖌️ **MS Paint & Grafik Tasarım Entegrasyonu**: Seçilen rengi MS Paint'in "Renkleri Düzenle" menüsüne doğrudan aktarmak için entegre görsel kılavuz.
- 🌐 **Çoklu Dil Desteği (i18n)**: Türkçe ve English arasında tek tıkla geçiş.
- ⚙️ **Gelişmiş Ayarlar**: Windows açılışında otomatik başlatma, kısayol tuşu yapılandırması ve dil ayarı.
- 📦 **Çift Dağıtım Paketi**:
  - **Portable (Taşınabilir)**: `ETDPicker_Portable.exe` (Kurulum gerektirmez).
  - **Setup (Kurulumlu)**: `ETDPicker_Setup.exe` (Sıfırdan Rust ile yazılmış özel iki dilli kurulum sihirbazı ve Denetim Masası uninstaller entegrasyonu).

---

## 🚀 Kısayollar (Hotkeys)

| Kısayol | Açıklama |
| :--- | :--- |
| **`Alt + P`** | Renk seçiciyi / büyüteci hemen açar (Ayarlardan değiştirilebilir) |
| **`Space` veya `Sol Tık`** | Büyüteçteyken rengi kilitler ve seçer |
| **`Esc`** | Seçimi iptal eder |
| **`Ctrl + C`** | Seçili rengin HEX kodunu panoya kopyalar |

---

## 🖌️ MS Paint'te Renk Nasıl Kullanılır?

1. `Alt + P` ile ekrandaki rengi seçin.
2. ETDPicker ekranında görünen **R (Kırmızı)**, **G (Yeşil)**, **B (Mavi)** değerlerini inceleyin (veya **"RGB Kopyala"** butonuna basın).
3. **MS Paint**'i açıp üst menüden **"Renkleri Düzenle" (Edit Colors)** butonuna tıklayın.
4. Sağ alt köşedeki **Kırmızı, Yeşil, Mavi** kutucuklarına bu değerleri yazın ve **"Özel Renklere Ekle"** -> **"Tamam"** diyerek renginizi hemen kullanın.
5. Ayrıntılı rehber için [PAINT_GUIDE.md](docs/PAINT_GUIDE.md) dosyasına göz atabilirsiniz.

---

## 🛠️ Derleme ve Geliştirme (Build Instructions)

Projeyi kaynak koddan derlemek için bilgisayarınızda Rust kurulu olmalıdır:

```bash
# Bağımlılıkları kontrol et ve derle
cargo build --release

# Çıktılar target/release klasöründe oluşacaktır:
# - target/release/etd-picker.exe
# - target/release/etd-installer.exe
```

---

## 📄 Lisans ve Yasal Uyarılar (License & Legal)

- **Kod Tabanı**: [MIT License](LICENSE) altında açık kaynak olarak dağıtılmaktadır.
- **Fikri Mülkiyet**: **ETDPicker** proje adı ve resmi logosu (`logo.jpg`) **Emir Tuğra Dağ**'ın fikri mülkiyetidir.
- **Sorumluluk Reddi (Disclaimer)**: Bu yazılım "OLDUĞU GİBİ" (AS IS) sağlanmaktadır. Programın kullanımından kaynaklanabilecek her türlü doğrudan veya dolaylı durumdan kullanıcının kendisi mesuldür. Ayrıntılar için [DISCLAIMER.md](DISCLAIMER.md) dosyasını inceleyiniz.

---

<div align="center">
  Geliştirici / Yapımcı: <b>Emir Tuğra Dağ</b>
</div>
