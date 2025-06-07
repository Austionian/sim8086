use sim8086::disassemble;
use std::{env::current_dir, fs::File, io::Read};

#[test]
fn listing_37() {
    let mut file = File::open(format!(
        "{}/listing_0037_single_register_mov",
        current_dir().unwrap().display()
    ))
    .expect("file not found");

    let mut buffer = Vec::new();

    let _bytes = file.read_to_end(&mut buffer).expect("unable to read");

    assert_eq!(
        disassemble(buffer),
        r#"bits 16 

mov cx, bx
"#
    );
}

#[test]
fn listing_38() {
    let mut file = File::open(format!(
        "{}/listing_0038_many_register_mov",
        current_dir().unwrap().display()
    ))
    .expect("file not found");

    let mut buffer = Vec::new();

    let _bytes = file.read_to_end(&mut buffer).expect("unable to read");

    assert_eq!(
        disassemble(buffer),
        r#"bits 16 

mov cx, bx
mov ch, ah
mov dx, bx
mov si, bx
mov bx, di
mov al, cl
mov ch, ch
mov bx, ax
mov bx, si
mov sp, di
mov bp, ax
"#
    );
}

#[test]
fn listing_39() {
    let mut file = File::open(format!(
        "{}/listing_0039_more_movs",
        current_dir().unwrap().display()
    ))
    .expect("file not found");

    let mut buffer = Vec::new();

    let _bytes = file.read_to_end(&mut buffer).expect("unable to read");

    assert_eq!(
        disassemble(buffer),
        r#"bits 16 

mov si, bx
mov dh, al
mov cl, 12
mov ch, -12
mov cx, 12
mov cx, -12
mov dx, 3948
mov dx, -3948
mov al, [bx + si]
mov bx, [bp + di]
mov dx, [bp]
mov ah, [bx + si + 4]
mov al, [bx + si + 4999]
mov [bx + di], cx
mov [bp + si], cl
mov [bp], ch
"#
    )
}

#[test]
fn listing_40() {
    let mut file = File::open(format!(
        "{}/listing_0040_challenge_movs",
        current_dir().unwrap().display()
    ))
    .expect("file not found");

    let mut buffer = Vec::new();

    let _bytes = file.read_to_end(&mut buffer).expect("unable to read");

    assert_eq!(
        disassemble(buffer),
        r#"bits 16 

mov ax, [bx + di - 37]
mov [si - 300], cx
mov dx, [bx - 32]
mov [bp + di], byte 7
mov [di + 901], word 347
mov bp, [5]
mov bx, [3458]
mov ax, [2555]
mov ax, [16]
mov [2554], ax
mov [15], ax
"#
    )
}
