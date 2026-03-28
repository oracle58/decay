use std::io::{self, Stdout, Write, stdout};

const ATTR_BOLD: u8 = 1;
const ATTR_DIM: u8 = 2;
const ATTR_ITALIC: u8 = 4;
const ATTR_UNDERLINE: u8 = 8;
const ATTR_STRIKETHROUGH: u8 = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    ch: char,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    attrs: u8,
}

impl Cell {
    fn blank() -> Self {
        Self { ch: ' ', fg: (255, 255, 255), bg: (0, 0, 0), attrs: 0 }
    }
}

/// Double-buffered terminal with differential rendering.
pub struct Term {
    out: Stdout,
    raw_buf: Vec<u8>,
    front: Vec<Cell>,
    back: Vec<Cell>,
    cols: u16,
    rows: u16,
    cx: u16,
    cy: u16,
    cur_fg: (u8, u8, u8),
    cur_bg: (u8, u8, u8),
    cur_attrs: u8,
    first_frame: bool,
    clip: Option<(u16, u16, u16, u16)>,
}

impl Term {
    pub fn new() -> io::Result<Self> {
        platform::init()?;
        let (cols, rows) = platform::term_size()
            .ok()
            .filter(|&(c, r)| c > 0 && r > 0)
            .unwrap_or((80, 24));
        let n = (cols as usize) * (rows as usize);
        Ok(Self {
            out: stdout(),
            raw_buf: Vec::with_capacity(512),
            front: vec![Cell::blank(); n],
            back: vec![Cell::blank(); n],
            cols,
            rows,
            cx: 0,
            cy: 0,
            cur_fg: (255, 255, 255),
            cur_bg: (0, 0, 0),
            cur_attrs: 0,
            first_frame: true,
            clip: None,
        })
    }

    pub fn size() -> io::Result<(u16, u16)> {
        platform::term_size()
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn clear(&mut self) -> &mut Self {
        // Event-driven resize: only re-query size when the platform signals a change
        if platform::take_resize_flag() {
            if let Ok((cols, rows)) = Self::size() {
                if cols != self.cols || rows != self.rows {
                    self.cols = cols;
                    self.rows = rows;
                    let n = (cols as usize) * (rows as usize);
                    self.front = vec![Cell::blank(); n];
                    self.back = vec![Cell::blank(); n];
                    self.first_frame = true;
                }
            }
        }
        self.back.fill(Cell::blank());
        self.cx = 0;
        self.cy = 0;
        self.cur_fg = (255, 255, 255);
        self.cur_bg = (0, 0, 0);
        self.cur_attrs = 0;
        self.clip = None;
        self
    }

    pub fn set_clip(&mut self, x: u16, y: u16, w: u16, h: u16) -> &mut Self {
        self.clip = Some((x, y, w, h));
        self
    }

    pub fn clear_clip(&mut self) -> &mut Self {
        self.clip = None;
        self
    }

    pub fn move_to(&mut self, col: u16, row: u16) -> &mut Self {
        self.cx = col;
        self.cy = row;
        self
    }

    pub fn print(&mut self, s: &str) -> &mut Self {
        for ch in s.chars() {
            self.put_cell(ch);
        }
        self
    }

    pub fn print_char(&mut self, c: char) -> &mut Self {
        self.put_cell(c);
        self
    }

    pub fn print_n(&mut self, c: char, n: usize) -> &mut Self {
        for _ in 0..n {
            self.put_cell(c);
        }
        self
    }

    fn put_cell(&mut self, ch: char) {
        if self.cx < self.cols && self.cy < self.rows {
            if let Some((clip_x, clip_y, clip_w, clip_h)) = self.clip {
                if self.cx < clip_x || self.cx >= clip_x + clip_w
                    || self.cy < clip_y || self.cy >= clip_y + clip_h
                {
                    self.cx += 1;
                    return;
                }
            }
            let idx = (self.cy as usize) * (self.cols as usize) + (self.cx as usize);
            self.back[idx] = Cell { ch, fg: self.cur_fg, bg: self.cur_bg, attrs: self.cur_attrs };
            self.cx += 1;
        }
    }

    pub fn fg_rgb(&mut self, r: u8, g: u8, b: u8) -> &mut Self {
        self.cur_fg = (r, g, b);
        self
    }

    pub fn bg_rgb(&mut self, r: u8, g: u8, b: u8) -> &mut Self {
        self.cur_bg = (r, g, b);
        self
    }

    pub fn bold(&mut self) -> &mut Self {
        self.cur_attrs |= ATTR_BOLD;
        self
    }

    pub fn dim(&mut self) -> &mut Self {
        self.cur_attrs |= ATTR_DIM;
        self
    }

    pub fn italic(&mut self) -> &mut Self {
        self.cur_attrs |= ATTR_ITALIC;
        self
    }

    pub fn underline(&mut self) -> &mut Self {
        self.cur_attrs |= ATTR_UNDERLINE;
        self
    }

    pub fn strikethrough(&mut self) -> &mut Self {
        self.cur_attrs |= ATTR_STRIKETHROUGH;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.cur_fg = (255, 255, 255);
        self.cur_bg = (0, 0, 0);
        self.cur_attrs = 0;
        self
    }

    pub fn hide_cursor(&mut self) -> &mut Self {
        self.raw_buf.extend_from_slice(b"\x1b[?25l");
        self
    }

    pub fn show_cursor(&mut self) -> &mut Self {
        self.raw_buf.extend_from_slice(b"\x1b[?25h");
        self
    }

    pub fn enter_alt_screen(&mut self) -> &mut Self {
        self.raw_buf.extend_from_slice(b"\x1b[?1049h");
        self
    }

    pub fn leave_alt_screen(&mut self) -> &mut Self {
        self.raw_buf.extend_from_slice(b"\x1b[?1049l");
        self
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.first_frame {
            self.raw_buf.extend_from_slice(b"\x1b[2J");
        }
        self.diff_emit();
        if !self.raw_buf.is_empty() {
            self.out.write_all(&self.raw_buf)?;
            self.out.flush()?;
            self.raw_buf.clear();
        }
        Ok(())
    }

    fn diff_emit(&mut self) {
        let mut out: Vec<u8> = Vec::with_capacity(4096);
        let mut last_fg = (0u8, 0u8, 0u8);
        let mut last_bg = (0u8, 0u8, 0u8);
        let mut last_attrs: u8 = 0xFF;
        let mut crow: i32 = -1;
        let mut ccol: i32 = -1;
        let force = self.first_frame;

        for row in 0..self.rows {
            for col in 0..self.cols {
                let idx = (row as usize) * (self.cols as usize) + (col as usize);
                let b = self.back[idx];

                if !force && b == self.front[idx] {
                    continue;
                }

                // Cursor move (skip if already in position)
                if crow != row as i32 || ccol != col as i32 {
                    let _ = write!(out, "\x1b[{};{}H", row + 1, col + 1);
                }

                // Attribute changes
                if b.attrs != last_attrs {
                    out.extend_from_slice(b"\x1b[0m");
                    if b.attrs & ATTR_BOLD != 0 {
                        out.extend_from_slice(b"\x1b[1m");
                    }
                    if b.attrs & ATTR_DIM != 0 {
                        out.extend_from_slice(b"\x1b[2m");
                    }
                    if b.attrs & ATTR_ITALIC != 0 {
                        out.extend_from_slice(b"\x1b[3m");
                    }
                    if b.attrs & ATTR_UNDERLINE != 0 {
                        out.extend_from_slice(b"\x1b[4m");
                    }
                    if b.attrs & ATTR_STRIKETHROUGH != 0 {
                        out.extend_from_slice(b"\x1b[9m");
                    }
                    last_fg = (0, 0, 0);
                    last_bg = (0, 0, 0);
                    last_attrs = b.attrs;
                }
                if b.fg != last_fg {
                    let _ = write!(out, "\x1b[38;2;{};{};{}m", b.fg.0, b.fg.1, b.fg.2);
                    last_fg = b.fg;
                }
                if b.bg != last_bg {
                    let _ = write!(out, "\x1b[48;2;{};{};{}m", b.bg.0, b.bg.1, b.bg.2);
                    last_bg = b.bg;
                }

                let mut tmp = [0u8; 4];
                out.extend_from_slice(b.ch.encode_utf8(&mut tmp).as_bytes());

                ccol = col as i32 + 1;
                crow = row as i32;
            }
        }

        if !out.is_empty() {
            out.extend_from_slice(b"\x1b[0m");
            self.raw_buf.extend_from_slice(&out);
        }

        self.front.copy_from_slice(&self.back);
        self.first_frame = false;
    }
}

/// Detach from the parent console and open a dedicated window.
///
/// On Windows, calls FreeConsole + AllocConsole so all subsequent terminal I/O
/// targets the new window. The parent shell gets its prompt back immediately.
/// No-op on non-Windows platforms.
pub fn spawn_window() {
    #[cfg(windows)]
    {
        unsafe extern "system" {
            fn FreeConsole() -> i32;
            fn AllocConsole() -> i32;
            fn SetConsoleTitleW(title: *const u16) -> i32;
        }

        unsafe {
            FreeConsole();
            AllocConsole();
            let title: Vec<u16> = "decay\0".encode_utf16().collect();
            SetConsoleTitleW(title.as_ptr());
        }
    }
}

pub use platform::cleanup;
pub use platform::enable_raw_mode;
pub use platform::poll_input;

// Windows

#[cfg(windows)]
pub(crate) mod platform {
    use crate::core::input::{KeyCode, MouseButton, MouseEvent, MouseKind};
    use std::io;
    use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};

    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
    const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
    const ENABLE_LINE_INPUT: u32 = 0x0002;
    const ENABLE_ECHO_INPUT: u32 = 0x0004;
    const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
    const KEY_EVENT: u16 = 0x0001;
    const MOUSE_EVENT: u16 = 0x0002;
    const WINDOW_BUFFER_SIZE_EVENT: u16 = 0x0004;
    const SHIFT_PRESSED: u32 = 0x0010;
    const ENABLE_WINDOW_INPUT: u32 = 0x0008;
    const ENABLE_MOUSE_INPUT: u32 = 0x0010;
    const FROM_LEFT_1ST_BUTTON_PRESSED: u32 = 0x0001;
    const RIGHTMOST_BUTTON_PRESSED: u32 = 0x0002;
    const FROM_LEFT_2ND_BUTTON_PRESSED: u32 = 0x0004;

    static STDIN_HANDLE: AtomicIsize = AtomicIsize::new(0);
    static ORIGINAL_IN_MODE: AtomicU32 = AtomicU32::new(0);
    static RESIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Coord {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    struct SmallRect {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }

    #[repr(C)]
    struct ScreenBufferInfo {
        size: Coord,
        cursor: Coord,
        attributes: u16,
        window: SmallRect,
        max_size: Coord,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeyEventRecord {
        key_down: i32,
        repeat_count: u16,
        virtual_key_code: u16,
        virtual_scan_code: u16,
        uchar: u16,
        control_key_state: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct MouseEventRecord {
        mouse_position: Coord,
        button_state: u32,
        control_key_state: u32,
        event_flags: u32,
    }

    #[repr(C)]
    union EventUnion {
        key: KeyEventRecord,
        mouse: MouseEventRecord,
        _pad: [u8; 16],
    }

    #[repr(C)]
    struct InputRecord {
        event_type: u16,
        _pad: u16,
        event: EventUnion,
    }

    unsafe extern "system" {
        fn GetStdHandle(id: u32) -> isize;
        fn GetConsoleMode(handle: isize, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: isize, mode: u32) -> i32;
        fn GetConsoleScreenBufferInfo(handle: isize, info: *mut ScreenBufferInfo) -> i32;
        fn GetNumberOfConsoleInputEvents(handle: isize, count: *mut u32) -> i32;
        fn ReadConsoleInputW(
            handle: isize,
            buffer: *mut InputRecord,
            length: u32,
            read: *mut u32,
        ) -> i32;
    }

    pub fn init() -> io::Result<()> {
        unsafe {
            let h = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode = 0u32;
            if GetConsoleMode(h, &mut mode) == 0 {
                return Err(io::Error::last_os_error());
            }
            if mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
                if SetConsoleMode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == 0 {
                    return Err(io::Error::last_os_error());
                }
            }
        }
        Ok(())
    }

    pub fn enable_raw_mode() -> io::Result<()> {
        unsafe {
            let h = GetStdHandle(STD_INPUT_HANDLE);
            let mut mode: u32 = 0;
            if GetConsoleMode(h, &mut mode) == 0 {
                return Err(io::Error::last_os_error());
            }
            STDIN_HANDLE.store(h, Ordering::SeqCst);
            ORIGINAL_IN_MODE.store(mode, Ordering::SeqCst);
            let raw = (mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT))
                | ENABLE_WINDOW_INPUT
                | ENABLE_MOUSE_INPUT;
            if SetConsoleMode(h, raw) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn take_resize_flag() -> bool {
        RESIZED.swap(false, Ordering::SeqCst)
    }

    pub fn cleanup() {
        unsafe {
            let h = STDIN_HANDLE.load(Ordering::SeqCst);
            let mode = ORIGINAL_IN_MODE.load(Ordering::SeqCst);
            if h != 0 {
                SetConsoleMode(h, mode);
            }
        }
    }

    pub fn term_size() -> io::Result<(u16, u16)> {
        unsafe {
            let h = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut info = core::mem::zeroed::<ScreenBufferInfo>();
            if GetConsoleScreenBufferInfo(h, &mut info) == 0 {
                return Err(io::Error::last_os_error());
            }
            let cols = (info.window.right - info.window.left + 1) as u16;
            let rows = (info.window.bottom - info.window.top + 1) as u16;
            Ok((cols, rows))
        }
    }

    pub fn poll_input() -> (Vec<KeyCode>, Vec<MouseEvent>) {
        let mut keys = Vec::new();
        let mut mouse = Vec::new();
        let h = STDIN_HANDLE.load(Ordering::SeqCst);
        if h == 0 {
            return (keys, mouse);
        }
        unsafe {
            let mut count: u32 = 0;
            if GetNumberOfConsoleInputEvents(h, &mut count) == 0 || count == 0 {
                return (keys, mouse);
            }
            let mut records: Vec<InputRecord> = Vec::with_capacity(count as usize);
            records.set_len(count as usize);
            std::ptr::write_bytes(records.as_mut_ptr(), 0, count as usize);
            let mut read: u32 = 0;
            if ReadConsoleInputW(h, records.as_mut_ptr(), count, &mut read) == 0 {
                return (keys, mouse);
            }
            for record in &records[..read as usize] {
                match record.event_type {
                    KEY_EVENT => {
                        let key = record.event.key;
                        if key.key_down != 0 {
                            if let Some(kc) = vk_to_keycode(
                                key.virtual_key_code,
                                key.uchar,
                                key.control_key_state,
                            ) {
                                keys.push(kc);
                            }
                        }
                    }
                    MOUSE_EVENT => {
                        let me = record.event.mouse;
                        let col = me.mouse_position.x as u16;
                        let row = me.mouse_position.y as u16;
                        let flags = me.event_flags;
                        let btn = me.button_state;

                        let kind = if flags & 0x04 != 0 {
                            // Mouse wheel
                            if (btn >> 16) as i16 > 0 {
                                MouseKind::ScrollUp
                            } else {
                                MouseKind::ScrollDown
                            }
                        } else if flags & 0x01 != 0 {
                            MouseKind::Move
                        } else if btn & FROM_LEFT_1ST_BUTTON_PRESSED != 0 {
                            MouseKind::Press(MouseButton::Left)
                        } else if btn & RIGHTMOST_BUTTON_PRESSED != 0 {
                            MouseKind::Press(MouseButton::Right)
                        } else if btn & FROM_LEFT_2ND_BUTTON_PRESSED != 0 {
                            MouseKind::Press(MouseButton::Middle)
                        } else {
                            MouseKind::Release(MouseButton::Left)
                        };

                        mouse.push(MouseEvent { kind, col, row });
                    }
                    WINDOW_BUFFER_SIZE_EVENT => {
                        RESIZED.store(true, Ordering::SeqCst);
                    }
                    _ => {}
                }
            }
        }
        (keys, mouse)
    }

    fn vk_to_keycode(vk: u16, uchar: u16, state: u32) -> Option<KeyCode> {
        match vk {
            0x0D => Some(KeyCode::Enter),
            0x1B => Some(KeyCode::Escape),
            0x09 => {
                if state & SHIFT_PRESSED != 0 {
                    Some(KeyCode::BackTab)
                } else {
                    Some(KeyCode::Tab)
                }
            }
            0x08 => Some(KeyCode::Backspace),
            0x26 => Some(KeyCode::Up),
            0x28 => Some(KeyCode::Down),
            0x25 => Some(KeyCode::Left),
            0x27 => Some(KeyCode::Right),
            0x20 => Some(KeyCode::Char(' ')),
            _ => {
                if uchar != 0 {
                    char::from_u32(uchar as u32).map(KeyCode::Char)
                } else {
                    None
                }
            }
        }
    }
}

// Unix

#[cfg(unix)]
pub(crate) mod platform {
    use crate::core::input::{KeyCode, MouseEvent};
    use std::io;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};

    static RESIZED: AtomicBool = AtomicBool::new(false);

    #[cfg(target_os = "linux")]
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Termios {
        c_iflag: u32,
        c_oflag: u32,
        c_cflag: u32,
        c_lflag: u32,
        c_line: u8,
        c_cc: [u8; 32],
        c_ispeed: u32,
        c_ospeed: u32,
    }

    #[cfg(target_os = "macos")]
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Termios {
        c_iflag: u64,
        c_oflag: u64,
        c_cflag: u64,
        c_lflag: u64,
        c_cc: [u8; 20],
        c_ispeed: u64,
        c_ospeed: u64,
    }

    #[repr(C)]
    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        ws_xpixel: u16,
        ws_ypixel: u16,
    }

    #[cfg(target_os = "linux")]
    const TIOCGWINSZ: core::ffi::c_ulong = 0x5413;
    #[cfg(target_os = "macos")]
    const TIOCGWINSZ: core::ffi::c_ulong = 0x4008_7468;

    #[cfg(target_os = "linux")]
    const ECHO: u32 = 0x0008;
    #[cfg(target_os = "linux")]
    const ICANON: u32 = 0x0002;
    #[cfg(target_os = "linux")]
    const ISIG: u32 = 0x0001;

    #[cfg(target_os = "macos")]
    const ECHO: u64 = 0x0000_0008;
    #[cfg(target_os = "macos")]
    const ICANON: u64 = 0x0000_0100;
    #[cfg(target_os = "macos")]
    const ISIG: u64 = 0x0000_0080;

    #[cfg(target_os = "linux")]
    const ICRNL: u32 = 0x0100;
    #[cfg(target_os = "linux")]
    const IXON: u32 = 0x0400;
    #[cfg(target_os = "linux")]
    const OPOST: u32 = 0x0001;
    #[cfg(target_os = "linux")]
    const IEXTEN: u32 = 0x8000;

    #[cfg(target_os = "macos")]
    const ICRNL: u64 = 0x0000_0100;
    #[cfg(target_os = "macos")]
    const IXON: u64 = 0x0000_0200;
    #[cfg(target_os = "macos")]
    const OPOST: u64 = 0x0000_0001;
    #[cfg(target_os = "macos")]
    const IEXTEN: u64 = 0x0000_0400;

    #[cfg(target_os = "linux")]
    const VMIN: usize = 6;
    #[cfg(target_os = "linux")]
    const VTIME: usize = 5;

    #[cfg(target_os = "macos")]
    const VMIN: usize = 16;
    #[cfg(target_os = "macos")]
    const VTIME: usize = 17;

    const O_NONBLOCK: core::ffi::c_int = 0x0800;
    #[cfg(target_os = "macos")]
    const O_NONBLOCK_MAC: core::ffi::c_int = 0x0004;

    const F_GETFL: core::ffi::c_int = 3;
    const F_SETFL: core::ffi::c_int = 4;

    const SIGWINCH: core::ffi::c_int = 28;

    #[repr(C)]
    struct Sigaction {
        sa_handler: usize,
        sa_mask: [u64; 2], // large enough for both linux (128 bytes) and macos (4 bytes)
        sa_flags: core::ffi::c_int,
        #[cfg(target_os = "linux")]
        sa_restorer: usize,
    }

    unsafe extern "C" {
        fn ioctl(fd: core::ffi::c_int, request: core::ffi::c_ulong, ...) -> core::ffi::c_int;
        fn tcgetattr(fd: core::ffi::c_int, termios: *mut Termios) -> core::ffi::c_int;
        fn tcsetattr(
            fd: core::ffi::c_int,
            action: core::ffi::c_int,
            termios: *const Termios,
        ) -> core::ffi::c_int;
        fn read(fd: core::ffi::c_int, buf: *mut u8, count: usize) -> isize;
        fn fcntl(fd: core::ffi::c_int, cmd: core::ffi::c_int, ...) -> core::ffi::c_int;
        fn sigaction(
            sig: core::ffi::c_int,
            act: *const Sigaction,
            oldact: *mut Sigaction,
        ) -> core::ffi::c_int;
    }

    extern "C" fn handle_sigwinch(_sig: core::ffi::c_int) {
        RESIZED.store(true, Ordering::SeqCst);
    }

    static ORIGINAL_TERMIOS: OnceLock<Termios> = OnceLock::new();

    pub fn init() -> io::Result<()> {
        Ok(())
    }

    pub fn enable_raw_mode() -> io::Result<()> {
        unsafe {
            let mut original = core::mem::zeroed::<Termios>();
            if tcgetattr(0, &mut original) != 0 {
                return Err(io::Error::last_os_error());
            }
            let _ = ORIGINAL_TERMIOS.set(original);

            let mut raw = original;
            raw.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
            raw.c_oflag &= !OPOST;
            raw.c_iflag &= !(ICRNL | IXON);
            raw.c_cc[VMIN] = 0;
            raw.c_cc[VTIME] = 0;

            if tcsetattr(0, 0, &raw) != 0 {
                return Err(io::Error::last_os_error());
            }

            let flags = fcntl(0, F_GETFL);
            #[cfg(target_os = "linux")]
            let nb = O_NONBLOCK;
            #[cfg(target_os = "macos")]
            let nb = O_NONBLOCK_MAC;
            fcntl(0, F_SETFL, flags | nb);

            // Register SIGWINCH handler for event-driven resize
            let mut sa: Sigaction = core::mem::zeroed();
            sa.sa_handler = handle_sigwinch as *const () as usize;
            sigaction(SIGWINCH, &sa, std::ptr::null_mut());
        }
        Ok(())
    }

    pub fn take_resize_flag() -> bool {
        RESIZED.swap(false, Ordering::SeqCst)
    }

    pub fn cleanup() {
        if let Some(original) = ORIGINAL_TERMIOS.get() {
            unsafe {
                tcsetattr(0, 0, original);
                let flags = fcntl(0, F_GETFL);
                #[cfg(target_os = "linux")]
                let nb = O_NONBLOCK;
                #[cfg(target_os = "macos")]
                let nb = O_NONBLOCK_MAC;
                fcntl(0, F_SETFL, flags & !nb);
            }
        }
    }

    pub fn term_size() -> io::Result<(u16, u16)> {
        unsafe {
            let mut ws = core::mem::zeroed::<Winsize>();
            if ioctl(1, TIOCGWINSZ, &mut ws) == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok((ws.ws_col, ws.ws_row))
        }
    }

    pub fn poll_input() -> (Vec<KeyCode>, Vec<MouseEvent>) {
        let mut keys = Vec::new();
        let mouse = Vec::new();
        let mut buf = [0u8; 128];
        let n = unsafe { read(0, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 {
            return (keys, mouse);
        }
        let bytes = &buf[..n as usize];
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == 0x1b {
                if i + 2 < bytes.len() && bytes[i + 1] == b'[' {
                    match bytes[i + 2] {
                        b'A' => {
                            keys.push(KeyCode::Up);
                            i += 3;
                        }
                        b'B' => {
                            keys.push(KeyCode::Down);
                            i += 3;
                        }
                        b'C' => {
                            keys.push(KeyCode::Right);
                            i += 3;
                        }
                        b'D' => {
                            keys.push(KeyCode::Left);
                            i += 3;
                        }
                        b'Z' => {
                            keys.push(KeyCode::BackTab);
                            i += 3;
                        }
                        _ => {
                            // Skip unknown escape sequence
                            i += 3;
                            while i < bytes.len()
                                && !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'~')
                            {
                                i += 1;
                            }
                            if i < bytes.len() {
                                i += 1;
                            }
                        }
                    }
                } else {
                    keys.push(KeyCode::Escape);
                    i += 1;
                }
            } else {
                match bytes[i] {
                    b'\r' | b'\n' => keys.push(KeyCode::Enter),
                    b'\t' => keys.push(KeyCode::Tab),
                    0x7f => keys.push(KeyCode::Backspace),
                    0x08 => keys.push(KeyCode::Backspace),
                    c if c >= 0x20 && c < 0x7f => keys.push(KeyCode::Char(c as char)),
                    _ => {}
                }
                i += 1;
            }
        }
        (keys, mouse)
    }
}

// Fallback

#[cfg(not(any(windows, unix)))]
pub(crate) mod platform {
    use crate::core::input::{KeyCode, MouseEvent};
    use std::io;

    pub fn init() -> io::Result<()> {
        Ok(())
    }

    pub fn enable_raw_mode() -> io::Result<()> {
        Ok(())
    }

    pub fn cleanup() {}

    pub fn take_resize_flag() -> bool {
        false
    }

    pub fn term_size() -> io::Result<(u16, u16)> {
        Ok((80, 24))
    }

    pub fn poll_input() -> (Vec<KeyCode>, Vec<MouseEvent>) {
        (Vec::new(), Vec::new())
    }
}
