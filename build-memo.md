Chạy tại thư mục project:

```powershell
cd D:\my-src\10.Developing\book-library
npm install
```

Chạy bản dev có hot reload:

```powershell
npm run tauri dev
```

Build release chỉ tạo file chạy:

```powershell
npm run tauri build -- --no-bundle
```

File kết quả:

```text
src-tauri\target\release\book-library.exe
```

Build release kèm bộ cài Windows:

```powershell
npm run tauri build
```

Bộ cài nằm trong:

```text
src-tauri\target\release\bundle\
```

Riêng OCR cần cài Tesseract có language data `jpn` và `eng`. Nếu Tesseract không nằm trong `PATH`, đặt biến môi trường trước khi chạy:

```powershell
$env:BOOK_LIBRARY_TESSERACT = "C:\Program Files\Tesseract-OCR\tesseract.exe"
npm run tauri dev
```

Chỉ build frontend web để kiểm tra TypeScript/UI:

```powershell
npm run build
```

Nhưng không nên dùng `npm run dev` để thử đầy đủ app vì các tính năng filesystem, SQLite và OCR cần chạy bên trong Tauri.