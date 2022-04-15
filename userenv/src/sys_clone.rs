#![no_std]
#![no_main]

mod syscall;
mod runtime;
mod console;

use syscall::*;
use core::slice::from_raw_parts_mut;
use core::mem::size_of;
use core::assert;

fn main() {
    assert!(syscall_chdir("/\0") == 0);
    assert!(syscall_openat(AT_FDCWD, "/clone\0", OpenFlags::CREATE, FileMode::empty()) > 0);
    let mut pipe: [i32; 2] = [0, 0];
    assert!(syscall_pipe(&mut pipe) == 0);
    println!("pipe({}, {})", pipe[0], pipe[1]);
    let forkret = syscall_clone(CloneFlags::empty(), 0 as usize as *const u8, 0, 0, 0);
    if forkret > 0 {
        assert!(syscall_close(pipe[0]) == 0);
        let buf = "100 101 102 103 104 105 106 107 108 109 110 111 112 113 114 115 116 117 118 119 120 121 122 123 124 125 126 127 128 129 130 131 132 133 134 135 136 137 138 139 140 141 142 143 144 145 146 147 148 149 150 151 152 153 154 155 156 157 158 159 160 161 162 163 164 165 166 167 168 169 170 171 172 173 174 175 176 177 178 179 180 181 182 183 184 185 186 187 188 189 190 191 192 193 194 195 196 197 198 199 200 201 202 203 204 205 206 207 208 209 210 211 212 213 214 215 216 217 218 219 220 221 222 223 224 225 226 227 228 229 230 231 232 233 234 235 236 237 238 239 240 241 242 243 244 245 246 247 248 249 250 251 252 253 254 255 256 257 258 259 260 261 262 263 264 265 266 267 268 269 270 271 272 273 274 275 276 277 278 279 280 281 282 283 284 285 286 287 288 289 290 291 292 293 294 295 296 297 298 299 ";
        syscall_write(pipe[1], buf.as_bytes());
    } else {
        assert!(syscall_close(pipe[1]) == 0);
        println!("child going");
        let mut buf: [u8; 64] = [0; 64];
        while syscall_read(pipe[0], &mut buf) > 0 {
            println!("{}", unsafe {core::str::from_utf8_unchecked(&buf)});
            buf = [0; 64];
        }
        assert!(syscall_close(pipe[0]) == 0);
    }
}