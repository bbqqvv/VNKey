# VNKey - Bộ Gõ Tiếng Việt Thế Hệ Mới 🚀

VNKey là một bộ gõ tiếng Việt hiện đại, kết hợp sức mạnh hiệu năng của **Rust** và giao diện thân thiện của **WPF (.NET 9)**. Được thiết kế để thay thế các bộ gõ cũ kỹ, VNKey tập trung vào tốc độ, độ chính xác và khả năng tùy biến cao.

---

## 📸 Hình ảnh Minh họa (Beta)

<p align="center">
  <img src="image.png" width="45%" />
  <img src="image-1.png" width="45%" />
</p>
<p align="center">
  <img src="image-2.png" width="45%" />
  <img src="image-3.png" width="45%" />
</p>

---

## ✨ Điểm Nổi Bật

- 🦀 **Lõi Rust (Core Engine):** Xử lý ngôn ngữ cực nhanh, an toàn bộ nhớ và không chiếm dụng tài nguyên hệ thống (Zero-overhead).
- ⌨️ **Bắt phím Cấp thấp:** Sử dụng Low-level Keyboard Hook để xử lý phím trực tiếp, đảm bảo độ trễ thấp nhất.
- 🎨 **Giao diện Modern:** Thiết kế theo phong cách tối giản, hỗ trợ Dark Mode/Light Mode đồng bộ với Windows.
- ⚙️ **Thông minh & Linh hoạt:**
  - Tự động nhận diện Tiếng Anh (Smart Literal Mode).
  - Kiểm tra chính tả dựa trên cấu trúc ngữ âm tiếng Việt.
  - Tự động viết hoa sau dấu chấm và phím Enter.
  - Phím tắt chuyển đổi E/V có thể tùy chỉnh hoàn toàn (Record Hotkey).
  - Tích hợp âm thanh Beep khi chuyển trạng thái.

---

## 🏗️ Cấu Trúc Dự Án

VNKey được xây dựng theo kiến trúc tách biệt giữa xử lý logic và giao diện:

- **`core/`**: Thư viện xử lý tiếng Việt viết bằng Rust. Đây là "bộ não" của ứng dụng, đảm nhận việc phân tích vần và bỏ dấu.
- **`platforms/windows/VNKey.Windows/`**: Ứng dụng GUI viết bằng C# WPF. Đảm nhận việc hiển thị, cài đặt và tương tác với hệ điều hành Windows.

---

## 🛠️ Hướng Dẫn Cài Đặt & Phát Triển

### Yêu Cầu Hệ Thống
- Windows 10/11
- [Rust Toolchain](https://rustup.rs/) (v1.75+)
- [.NET 9 SDK](https://dotnet.microsoft.com/en-us/download/dotnet/9.0)

### Các Bước Build
1. **Clone Repo:**
   ```bash
   git clone https://github.com/bbqqvv/VNKey.git
   cd VNKey
   ```

2. **Build Rust Core:**
   ```bash
   cd core
   cargo build --release
   ```

3. **Thiết lập Native DLL:**
   Copy `core/target/release/vnkey_core.dll` vào thư mục `platforms/windows/VNKey.Windows/Native/`.

4. **Build & Run WPF App:**
   ```bash
   cd platforms/windows/VNKey.Windows
   dotnet run --configuration Release
   ```

---

## 📜 Giấy Phép
Dự án được phát triển bởi **Van Quoc Bui** và cung cấp dưới dạng mã nguồn mở. Vui lòng tôn trọng các quy định về bản quyền khi sử dụng và đóng góp cho dự án.

