# 🎨 ETDPicker - Paint & Harici Programlarda Renk Kullanım Kılavuzu

Bu kılavuz, **ETDPicker** ile ekrandan seçtiğiniz renkleri **MS Paint**, **Adobe Photoshop**, **Figma**, **Canva** veya web projelerinizde (HTML/CSS) nasıl kullanacağınızı adım adım anlatır.

---

## 🖌️ 1. MS Paint (Microsoft Paint) İçinde Kullanım

MS Paint, web sitelerinde kullanılan `#FF5733` gibi HEX kodları yerine renkleri **Kırmızı (Red)**, **Yeşil (Green)** ve **Mavi (Blue)** sayısal değerleriyle (0 - 255 arası) kabul eder.

### Adım Adım Paint'e Renk Aktarma:
1. **Rengi Seçin**:
   - `Alt + P` kısayoluna basın (veya ETDPicker arayüzündeki **"Renk Seç"** butonuna tıklayın).
   - Ekrandaki istediğiniz rengin üzerine gelin ve **Sol Tık** yapın veya `Space` tuşuna basın.
2. **Değerleri Görün veya Kopyalayın**:
   - ETDPicker penceresinde seçilen rengin RGB bileşenleri görünür:
     - **Kırmızı (R)**: Örn. `243`
     - **Yeşil (G)**: Örn. `156`
     - **Mavi (B)**: Örn. `18`
   - Dilerseniz **"RGB Kopyala"** butonuna basarak tüm değeri panoya alabilirsiniz.
3. **MS Paint'i Açın**:
   - Üst menü çubuğundaki Renkler bölümünde bulunan **"Renkleri Düzenle" (Edit Colors)** simgesine tıklayın.
4. **Değerleri Girin**:
   - Açılan palet penceresinin sağ alt köşesinde yer alan:
     - **Kırmızı (Red):** kutucuğuna ETDPicker'daki **R** değerini yazın.
     - **Yeşil (Green):** kutucuğuna ETDPicker'daki **G** değerini yazın.
     - **Mavi (Blue):** kutucuğuna ETDPicker'daki **B** değerini yazın.
5. **Rengi Kaydedin**:
   - **"Özel Renklere Ekle" (Add to Custom Colors)** butonuna tıklayın.
   - **Tamam** butonuna basarak pencereyi kapatın.
   - Artık fırça, dolgu kovası veya çizim araçlarınız bu renkle boyayacaktır!

---

## 💻 2. Web & CSS & HTML Projelerinde Kullanım

1. ETDPicker penceresindeki **"HEX Kopyala"** butonuna tıklayın (Örn: `#F39C12`).
2. CSS dosyanızda ilgili özelliğe yapıştırın:
```css
.my-button {
    background-color: #F39C12;
    color: #FFFFFF;
}
```

---

## 🎨 3. Adobe Photoshop / Illustrator / Figma İçinde Kullanım

- **Figma / Adobe XD**: Renk paletini açın ve ETDPicker'dan kopyaladığınız **HEX** kodunu `#` alanına yapıştırın.
- **Photoshop**: Araç çubuğundaki ön plan rengine çift tıklayın, alttaki `#` kutusuna kopyaladığınız HEX kodunu veya sağ taraftaki `R:`, `G:`, `B:` kutularına değerleri yazın.

---

## ⚡ Hızlı Kısayollar

| Kısayol | İşlem |
| :--- | :--- |
| **`Alt + P`** (Varsayılan) | Renk Seçiciyi / Büyüteci hemen aktif eder |
| **`Space` / `Sol Tık`** | Büyüteçteyken rengi kilitler / seçer |
| **`Esc`** | Renk seçimini iptal eder |
| **`Ctrl + C`** | Seçili rengin HEX kodunu kopyalar |
