[bits 16]
[org 0x7c00]

start:
    ; Limpiar pantalla
    mov ax, 0xb800
    mov es, ax
    mov di, 0
    mov cx, 2000
    mov ax, 0x0f20 ; Espacio blanco con color blanco
    rep stosw
    
    ; Escribir mensaje
    mov si, msg1
    mov di, 0
    call print_string
    
    mov si, msg2
    mov di, 160
    call print_string
    
    mov si, msg3
    mov di, 320
    call print_string
    
    mov si, msg4
    mov di, 480
    call print_string
    
    ; Loop infinito
    jmp $

print_string:
    lodsb
    cmp al, 0
    je done
    stosb
    mov al, 0x0f
    stosb
    jmp print_string
done:
    ret

msg1 db 'CRONOS W-OS v2.0 - Sistema Operativo Soberano', 0
msg2 db 'Exokernel con Grafos - IA Colmena - Crystal UI', 0
msg3 db 'Seguridad AEGIS - Graph Memory - Genesis Auto-creation', 0
msg4 db 'Kernel inicializado exitosamente - Sistema listo', 0

times 510-($-$$) db 0
dw 0xaa55
