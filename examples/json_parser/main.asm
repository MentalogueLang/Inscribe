.intel_syntax noprefix
.text
.global mainCRTStartup
mainCRTStartup:
    sub rsp, 40
    call __ml_fn_main
    add rsp, 40
    ret
__ml_fn_bool_to_int:
    sub rsp, 56
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_bool_to_int.Lbb0
__ml_fn_bool_to_int.Lbb0:
    mov rax, qword ptr [rsp + 40]
    cmp rax, 0
    jne __ml_fn_bool_to_int.Lbb1
    jmp __ml_fn_bool_to_int.Lbb2
__ml_fn_bool_to_int.Lbb1:
    mov rax, 1
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_bool_to_int.Lbb3
__ml_fn_bool_to_int.Lbb2:
    mov rax, 0
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_bool_to_int.Lbb3
__ml_fn_bool_to_int.Lbb3:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 56
    ret
__ml_fn_string_length:
    sub rsp, 176
    mov r10, rcx
    mov r11, rcx
__ml_fn_string_length.len_loop:
    movzx eax, byte ptr [r11]
    cmp rax, 0
    je __ml_fn_string_length.len_done
    add r11, 1
    jmp __ml_fn_string_length.len_loop
__ml_fn_string_length.len_done:
    mov rax, r11
    sub rax, r10
    add rsp, 176
    ret
__ml_fn_string_byte_at:
    sub rsp, 176
    mov r10, rcx
    mov r11, rdx
    cmp r11, 0
    jl __ml_fn_string_byte_at.byte_zero
__ml_fn_string_byte_at.byte_loop:
    cmp r11, 0
    je __ml_fn_string_byte_at.byte_at_index
    movzx eax, byte ptr [r10]
    cmp rax, 0
    je __ml_fn_string_byte_at.byte_zero
    add r10, 1
    sub r11, 1
    jmp __ml_fn_string_byte_at.byte_loop
__ml_fn_string_byte_at.byte_at_index:
    movzx eax, byte ptr [r10]
    cmp rax, 0
    je __ml_fn_string_byte_at.byte_zero
    add rsp, 176
    ret
__ml_fn_string_byte_at.byte_zero:
    mov rax, 0
    add rsp, 176
    ret
__ml_fn_len:
    sub rsp, 56
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_len.Lbb0
__ml_fn_len.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_string_length
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_len.Lbb1
__ml_fn_len.Lbb1:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 56
    ret
__ml_fn_byte_at:
    sub rsp, 72
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn_byte_at.Lbb0
__ml_fn_byte_at.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_string_byte_at
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_byte_at.Lbb1
__ml_fn_byte_at.Lbb1:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 72
    ret
__ml_fn_in_bounds:
    sub rsp, 88
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn_in_bounds.Lbb0
__ml_fn_in_bounds.Lbb0:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 0
    cmp rax, rcx
    setge al
    movzx eax, al
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn_in_bounds.Lbb1
    jmp __ml_fn_in_bounds.Lbb2
__ml_fn_in_bounds.Lbb1:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_len
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn_in_bounds.Lbb4
__ml_fn_in_bounds.Lbb2:
    mov rax, 0
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_in_bounds.Lbb3
__ml_fn_in_bounds.Lbb3:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 88
    ret
__ml_fn_in_bounds.Lbb4:
    mov rax, qword ptr [rsp + 48]
    mov rcx, qword ptr [rsp + 72]
    cmp rax, rcx
    setl al
    movzx eax, al
    mov qword ptr [rsp + 80], rax
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_in_bounds.Lbb3
__ml_fn_is_space:
    sub rsp, 104
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_is_space.Lbb0
__ml_fn_is_space.Lbb0:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 32
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 72], rax
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn_is_space.Lbb1
    jmp __ml_fn_is_space.Lbb2
__ml_fn_is_space.Lbb1:
    mov rax, 1
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn_is_space.Lbb3
__ml_fn_is_space.Lbb2:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 10
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 80], rax
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn_is_space.Lbb3
__ml_fn_is_space.Lbb3:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn_is_space.Lbb4
    jmp __ml_fn_is_space.Lbb5
__ml_fn_is_space.Lbb4:
    mov rax, 1
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_is_space.Lbb6
__ml_fn_is_space.Lbb5:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 13
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 88], rax
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_is_space.Lbb6
__ml_fn_is_space.Lbb6:
    mov rax, qword ptr [rsp + 56]
    cmp rax, 0
    jne __ml_fn_is_space.Lbb7
    jmp __ml_fn_is_space.Lbb8
__ml_fn_is_space.Lbb7:
    mov rax, 1
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_is_space.Lbb9
__ml_fn_is_space.Lbb8:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 9
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 96], rax
    mov rax, qword ptr [rsp + 96]
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_is_space.Lbb9
__ml_fn_is_space.Lbb9:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 104
    ret
__ml_fn_is_digit:
    sub rsp, 72
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_is_digit.Lbb0
__ml_fn_is_digit.Lbb0:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 48
    cmp rax, rcx
    setge al
    movzx eax, al
    mov qword ptr [rsp + 56], rax
    mov rax, qword ptr [rsp + 56]
    cmp rax, 0
    jne __ml_fn_is_digit.Lbb1
    jmp __ml_fn_is_digit.Lbb2
__ml_fn_is_digit.Lbb1:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 57
    cmp rax, rcx
    setle al
    movzx eax, al
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_is_digit.Lbb3
__ml_fn_is_digit.Lbb2:
    mov rax, 0
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_is_digit.Lbb3
__ml_fn_is_digit.Lbb3:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 72
    ret
__ml_fn_is_hex_digit:
    sub rsp, 120
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_is_hex_digit.Lbb0
__ml_fn_is_hex_digit.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_is_digit
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn_is_hex_digit.Lbb1
__ml_fn_is_hex_digit.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn_is_hex_digit.Lbb2
    jmp __ml_fn_is_hex_digit.Lbb3
__ml_fn_is_hex_digit.Lbb2:
    mov rax, 1
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_is_hex_digit.Lbb4
__ml_fn_is_hex_digit.Lbb3:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 65
    cmp rax, rcx
    setge al
    movzx eax, al
    mov qword ptr [rsp + 80], rax
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn_is_hex_digit.Lbb5
    jmp __ml_fn_is_hex_digit.Lbb6
__ml_fn_is_hex_digit.Lbb4:
    jmp __ml_fn_is_hex_digit.Lbb8
__ml_fn_is_hex_digit.Lbb5:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 70
    cmp rax, rcx
    setle al
    movzx eax, al
    mov qword ptr [rsp + 88], rax
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn_is_hex_digit.Lbb7
__ml_fn_is_hex_digit.Lbb6:
    mov rax, 0
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn_is_hex_digit.Lbb7
__ml_fn_is_hex_digit.Lbb7:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_is_hex_digit.Lbb4
__ml_fn_is_hex_digit.Lbb8:
    mov rax, 1
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_is_hex_digit.Lbb9
__ml_fn_is_hex_digit.Lbb9:
    mov rax, 1
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 120
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_invalid:
    sub rsp, 40
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_invalid.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_invalid.Lbb0:
    mov rax, 0
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 40
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_object:
    sub rsp, 40
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_object.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_object.Lbb0:
    mov rax, 1
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 40
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_array:
    sub rsp, 40
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_array.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_array.Lbb0:
    mov rax, 2
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 40
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_string:
    sub rsp, 40
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_string.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_string.Lbb0:
    mov rax, 3
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 40
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_bool:
    sub rsp, 40
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_bool.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_bool.Lbb0:
    mov rax, 5
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 40
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_null:
    sub rsp, 40
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_null.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_null.Lbb0:
    mov rax, 6
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 40
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result:
    sub rsp, 88
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result.Lbb0:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 10
    imul rax, rcx
    mov qword ptr [rsp + 56], rax
    mov rax, qword ptr [rsp + 56]
    mov rcx, qword ptr [rsp + 48]
    add rax, rcx
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 72], rax
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 88
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result:
    sub rsp, 72
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result.Lbb0:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 10
    imul rax, rcx
    mov qword ptr [rsp + 48], rax
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 56], rax
    mov rax, 0
    mov rcx, qword ptr [rsp + 56]
    sub rax, rcx
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 72
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded:
    sub rsp, 72
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb0:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 0
    cmp rax, rcx
    setl al
    movzx eax, al
    mov qword ptr [rsp + 56], rax
    mov rax, qword ptr [rsp + 56]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb1
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb1:
    mov rax, qword ptr [rsp + 40]
    neg rax
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb2:
    mov rax, qword ptr [rsp + 40]
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded.Lbb3:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 72
    ret
__ml_fn_result_ok:
    sub rsp, 56
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_result_ok.Lbb0
__ml_fn_result_ok.Lbb0:
    mov rax, qword ptr [rsp + 40]
    mov rcx, 0
    cmp rax, rcx
    setg al
    movzx eax, al
    mov qword ptr [rsp + 48], rax
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 56
    ret
__ml_fn_result_kind:
    sub rsp, 120
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_result_kind.Lbb0
__ml_fn_result_kind.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_result_kind.Lbb1
__ml_fn_result_kind.Lbb1:
    mov rax, qword ptr [rsp + 56]
    cmp rax, 0
    jne __ml_fn_result_kind.Lbb2
    jmp __ml_fn_result_kind.Lbb3
__ml_fn_result_kind.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn_result_kind.Lbb5
__ml_fn_result_kind.Lbb3:
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_invalid
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn_result_kind.Lbb6
__ml_fn_result_kind.Lbb4:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 120
    ret
__ml_fn_result_kind.Lbb5:
    mov rax, qword ptr [rsp + 72]
    mov rcx, 1
    sub rax, rcx
    mov qword ptr [rsp + 80], rax
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    mov rcx, 10
    cqo
    idiv rcx
    mov qword ptr [rsp + 88], rax
    mov rax, qword ptr [rsp + 88]
    mov rcx, 10
    imul rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rax, qword ptr [rsp + 64]
    mov rcx, qword ptr [rsp + 96]
    sub rax, rcx
    mov qword ptr [rsp + 104], rax
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_result_kind.Lbb4
__ml_fn_result_kind.Lbb6:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn_result_kind.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next:
    sub rsp, 72
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_encoded
    mov qword ptr [rsp + 48], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next.Lbb1:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    sub rax, rcx
    mov qword ptr [rsp + 56], rax
    mov rax, qword ptr [rsp + 56]
    mov rcx, 10
    cqo
    idiv rcx
    mov qword ptr [rsp + 64], rax
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 72
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte:
    sub rsp, 104
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    mov qword ptr [rsp + 56], r8
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb1:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb3:
    mov rax, 0
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb4:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 104
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb5:
    mov rax, qword ptr [rsp + 80]
    mov rcx, qword ptr [rsp + 56]
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 88], rax
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote:
    sub rsp, 104
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 34
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb2:
    mov rax, 1
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb3:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 92
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb4:
    mov rax, 1
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 104
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb5:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb6
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb6:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 88], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 88]
    mov r8, 34
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb7:
    mov rax, 0
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb8:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb9:
    mov rax, qword ptr [rsp + 96]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote:
    sub rsp, 88
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 34
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb2:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 72], rax
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb3:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 2
    add rax, rcx
    mov qword ptr [rsp + 80], rax
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote.Lbb4:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 88
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace:
    sub rsp, 120
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb1:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb3:
    mov rax, 0
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb4:
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb5:
    mov rcx, qword ptr [rsp + 80]
    call __ml_fn_is_space
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb6:
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb7:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace.Lbb8:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 120
    ret
__ml_fn_parse_document:
    sub rsp, 152
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_parse_document.Lbb0
__ml_fn_parse_document.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, 0
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn_parse_document.Lbb1
__ml_fn_parse_document.Lbb1:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 48], rax
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn_parse_document.Lbb2
__ml_fn_parse_document.Lbb2:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn_parse_document.Lbb3
    jmp __ml_fn_parse_document.Lbb4
__ml_fn_parse_document.Lbb3:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn_parse_document.Lbb6
__ml_fn_parse_document.Lbb4:
    mov rax, qword ptr [rsp + 48]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn_parse_document.Lbb5
__ml_fn_parse_document.Lbb5:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 152
    ret
__ml_fn_parse_document.Lbb6:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 88]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn_parse_document.Lbb7
__ml_fn_parse_document.Lbb7:
    mov rax, qword ptr [rsp + 96]
    mov qword ptr [rsp + 80], rax
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_len
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn_parse_document.Lbb8
__ml_fn_parse_document.Lbb8:
    mov rax, qword ptr [rsp + 80]
    mov rcx, qword ptr [rsp + 112]
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 120], rax
    mov rax, qword ptr [rsp + 120]
    cmp rax, 0
    jne __ml_fn_parse_document.Lbb9
    jmp __ml_fn_parse_document.Lbb10
__ml_fn_parse_document.Lbb9:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn_result_kind
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn_parse_document.Lbb12
__ml_fn_parse_document.Lbb10:
    mov rcx, qword ptr [rsp + 80]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn_parse_document.Lbb14
__ml_fn_parse_document.Lbb11:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn_parse_document.Lbb5
__ml_fn_parse_document.Lbb12:
    mov rcx, qword ptr [rsp + 80]
    mov rdx, qword ptr [rsp + 128]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn_parse_document.Lbb13
__ml_fn_parse_document.Lbb13:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn_parse_document.Lbb11
__ml_fn_parse_document.Lbb14:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn_parse_document.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value:
    sub rsp, 232
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb1:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb2:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb3
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb3:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb4:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 123
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb5:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 232
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb6:
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb7:
    mov rax, qword ptr [rsp + 104]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb8
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb8:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb9:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 91
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb10:
    mov rax, qword ptr [rsp + 96]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb11:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb12:
    mov rax, qword ptr [rsp + 128]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb13
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb13:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb14:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 116
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb15:
    mov rax, qword ptr [rsp + 120]
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb16:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb17:
    mov rax, qword ptr [rsp + 152]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb18
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb18:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb21
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb19:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 102
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 176], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb22
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb20:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb21:
    mov rax, qword ptr [rsp + 160]
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb22:
    mov rax, qword ptr [rsp + 176]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb23
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb24
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb23:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false
    mov qword ptr [rsp + 184], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb26
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb24:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 110
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 200], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb27
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb25:
    mov rax, qword ptr [rsp + 168]
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb26:
    mov rax, qword ptr [rsp + 184]
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb25
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb27:
    mov rax, qword ptr [rsp + 200]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb28
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb29
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb28:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null
    mov qword ptr [rsp + 208], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb31
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb29:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number
    mov qword ptr [rsp + 216], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb32
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb30:
    mov rax, qword ptr [rsp + 192]
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb25
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb31:
    mov rax, qword ptr [rsp + 208]
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb30
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb32:
    mov rax, qword ptr [rsp + 216]
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value.Lbb30
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string:
    sub rsp, 104
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb3:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb4:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 104
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb5:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 72]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb6:
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb7:
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body:
    sub rsp, 216
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_quote
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_advance_quote
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb3:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb4:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 216
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb5:
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_string
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb6:
    mov rcx, qword ptr [rsp + 72]
    mov rdx, qword ptr [rsp + 80]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb7:
    mov rax, qword ptr [rsp + 88]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb8:
    mov rax, qword ptr [rsp + 104]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb9
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb9:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb10:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 200], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb23
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb11:
    mov rax, qword ptr [rsp + 96]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb12:
    mov rax, qword ptr [rsp + 120]
    mov rcx, 92
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 128], rax
    mov rax, qword ptr [rsp + 128]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb13
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb13:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 136], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 136]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb14:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb15:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb16:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb17:
    mov rax, qword ptr [rsp + 160]
    mov rcx, 32
    cmp rax, rcx
    setl al
    movzx eax, al
    mov qword ptr [rsp + 168], rax
    mov rax, qword ptr [rsp + 168]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb18
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb18:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 176], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb21
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb19:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 184], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 184]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb22
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb20:
    mov rax, qword ptr [rsp + 152]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb21:
    mov rax, qword ptr [rsp + 176]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb22:
    mov rax, qword ptr [rsp + 192]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb23:
    mov rax, qword ptr [rsp + 200]
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape:
    sub rsp, 136
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb3:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb4:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 136
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb5:
    mov rax, qword ptr [rsp + 80]
    mov rcx, 117
    cmp rax, rcx
    sete al
    movzx eax, al
    mov qword ptr [rsp + 88], rax
    mov rax, qword ptr [rsp + 88]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb6
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb6:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 96]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb7:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 112], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 112]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb8:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb9:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb10:
    mov rax, qword ptr [rsp + 120]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb11:
    mov rax, qword ptr [rsp + 128]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_escape.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape:
    sub rsp, 264
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb0:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 3
    add rax, rcx
    mov qword ptr [rsp + 64], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 64]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb1:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb2:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb3:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 248], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb30
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb4:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 264
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb5:
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn_is_hex_digit
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb6:
    mov rax, qword ptr [rsp + 96]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb7
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb7:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 112], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 112]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb8:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 240], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb29
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb9:
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb10:
    mov rcx, qword ptr [rsp + 120]
    call __ml_fn_is_hex_digit
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb11:
    mov rax, qword ptr [rsp + 128]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb12
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb12:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 2
    add rax, rcx
    mov qword ptr [rsp + 144], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 144]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb13:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 232], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb28
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb14:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb15:
    mov rcx, qword ptr [rsp + 152]
    call __ml_fn_is_hex_digit
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb16:
    mov rax, qword ptr [rsp + 160]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb17
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb18
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb17:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 3
    add rax, rcx
    mov qword ptr [rsp + 176], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 176]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 184], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb18:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 224], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb27
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb19:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb20:
    mov rcx, qword ptr [rsp + 184]
    call __ml_fn_is_hex_digit
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb21
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb21:
    mov rax, qword ptr [rsp + 192]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb22
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb23
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb22:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 4
    add rax, rcx
    mov qword ptr [rsp + 200], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 200]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string_body
    mov qword ptr [rsp + 208], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb25
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb23:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 216], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb26
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb24:
    mov rax, qword ptr [rsp + 168]
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb25:
    mov rax, qword ptr [rsp + 208]
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb24
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb26:
    mov rax, qword ptr [rsp + 216]
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb24
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb27:
    mov rax, qword ptr [rsp + 224]
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb28:
    mov rax, qword ptr [rsp + 232]
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb29:
    mov rax, qword ptr [rsp + 240]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb30:
    mov rax, qword ptr [rsp + 248]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_unicode_escape.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true:
    sub rsp, 184
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 116
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb1:
    mov rax, qword ptr [rsp + 88]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb2:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 96]
    mov r8, 114
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb3:
    mov rax, 0
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb4:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb6
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb5:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb6:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 2
    add rax, rcx
    mov qword ptr [rsp + 112], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 112]
    mov r8, 117
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb7:
    mov rax, 0
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb8:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb10
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb9:
    mov rax, qword ptr [rsp + 120]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb10:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 3
    add rax, rcx
    mov qword ptr [rsp + 128], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 128]
    mov r8, 101
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb11:
    mov rax, 0
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb12:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb14
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb13:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb14:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 4
    add rax, rcx
    mov qword ptr [rsp + 144], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_bool
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb15:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb16:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 184
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb17:
    mov rcx, qword ptr [rsp + 144]
    mov rdx, qword ptr [rsp + 152]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb18
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb18:
    mov rax, qword ptr [rsp + 160]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb19:
    mov rax, qword ptr [rsp + 168]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_true.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false:
    sub rsp, 200
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 102
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb1:
    mov rax, qword ptr [rsp + 96]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb2:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 104], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 104]
    mov r8, 97
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb3:
    mov rax, 0
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb4:
    mov rax, qword ptr [rsp + 88]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb6
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb5:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb6:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 2
    add rax, rcx
    mov qword ptr [rsp + 120], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 120]
    mov r8, 108
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb7:
    mov rax, 0
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb8:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb10
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb9:
    mov rax, qword ptr [rsp + 128]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb10:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 3
    add rax, rcx
    mov qword ptr [rsp + 136], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 136]
    mov r8, 115
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb11:
    mov rax, 0
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb12:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb14
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb13:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb14:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 4
    add rax, rcx
    mov qword ptr [rsp + 152], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 152]
    mov r8, 101
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb15:
    mov rax, 0
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb16:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb18
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb17:
    mov rax, qword ptr [rsp + 160]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb18:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 5
    add rax, rcx
    mov qword ptr [rsp + 168], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_bool
    mov qword ptr [rsp + 176], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb21
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb19:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb23
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb20:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 200
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb21:
    mov rcx, qword ptr [rsp + 168]
    mov rdx, qword ptr [rsp + 176]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 184], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb22
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb22:
    mov rax, qword ptr [rsp + 184]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb23:
    mov rax, qword ptr [rsp + 192]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_false.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null:
    sub rsp, 184
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 110
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb1:
    mov rax, qword ptr [rsp + 88]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb2:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 96]
    mov r8, 117
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb3:
    mov rax, 0
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb4:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb6
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb5:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb6:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 2
    add rax, rcx
    mov qword ptr [rsp + 112], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 112]
    mov r8, 108
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb7:
    mov rax, 0
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb8:
    mov rax, qword ptr [rsp + 72]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb10
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb9:
    mov rax, qword ptr [rsp + 120]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb10:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 3
    add rax, rcx
    mov qword ptr [rsp + 128], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 128]
    mov r8, 108
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb11:
    mov rax, 0
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb12:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb14
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb13:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb14:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 4
    add rax, rcx
    mov qword ptr [rsp + 144], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_null
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb15:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb16:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 184
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb17:
    mov rcx, qword ptr [rsp + 144]
    mov rdx, qword ptr [rsp + 152]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb18
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb18:
    mov rax, qword ptr [rsp + 160]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb19:
    mov rax, qword ptr [rsp + 168]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_null.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number:
    sub rsp, 216
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    mov r8, 45
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb1:
    mov rax, qword ptr [rsp + 64]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb2
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb3
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb2:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 88], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 88]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 96], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb3:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_in_bounds
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb4:
    mov rax, qword ptr [rsp + 56]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 216
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb5:
    mov rax, qword ptr [rsp + 96]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb6
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb6:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 104], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 104]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb7:
    mov rax, 0
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb8:
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb9:
    mov rcx, qword ptr [rsp + 112]
    call __ml_fn_is_digit
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb10:
    mov rax, qword ptr [rsp + 120]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb11:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb12:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb13:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb14:
    mov rax, qword ptr [rsp + 168]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb15
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb15:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn_byte_at
    mov qword ptr [rsp + 176], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb18
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb16:
    mov rax, 0
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb17:
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb18:
    mov rcx, qword ptr [rsp + 176]
    call __ml_fn_is_digit
    mov qword ptr [rsp + 184], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb19:
    mov rax, qword ptr [rsp + 184]
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb20:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 208], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb22
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb21:
    mov rax, qword ptr [rsp + 152]
    mov qword ptr [rsp + 56], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb22:
    mov rax, qword ptr [rsp + 208]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_number.Lbb21
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array:
    sub rsp, 184
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb0:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 64], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 64]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb1:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 93
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb2:
    mov rax, qword ptr [rsp + 88]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb3
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb3:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_array
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb4:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb5:
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 184
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb6:
    mov rcx, qword ptr [rsp + 96]
    mov rdx, qword ptr [rsp + 104]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb7:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb8:
    mov rax, qword ptr [rsp + 128]
    mov qword ptr [rsp + 120], rax
    mov rcx, qword ptr [rsp + 120]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb9:
    mov rax, qword ptr [rsp + 144]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb10
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb10:
    mov rcx, qword ptr [rsp + 120]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb11:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb12:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb13:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 152]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb14:
    mov rax, qword ptr [rsp + 160]
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb15:
    mov rax, qword ptr [rsp + 168]
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value:
    sub rsp, 200
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb1:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 44
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb2:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb3
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb3:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 96]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb4:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 93
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb5:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 200
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb6:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 88], rax
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb7:
    mov rax, qword ptr [rsp + 120]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb8
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb8:
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb9:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb10:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb11:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 128]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb12:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb13:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb14:
    mov rax, qword ptr [rsp + 160]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb15
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb15:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 168], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_array
    mov qword ptr [rsp + 176], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb18
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb16:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb17:
    mov rax, qword ptr [rsp + 152]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb18:
    mov rcx, qword ptr [rsp + 168]
    mov rdx, qword ptr [rsp + 176]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 184], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb19:
    mov rax, qword ptr [rsp + 184]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb20:
    mov rax, qword ptr [rsp + 192]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_array_after_value.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object:
    sub rsp, 184
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb0:
    mov rax, qword ptr [rsp + 48]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 64], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 64]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb1:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 125
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb2:
    mov rax, qword ptr [rsp + 88]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb3
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb3:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_object
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb4:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb8
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb5:
    mov rax, qword ptr [rsp + 80]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 184
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb6:
    mov rcx, qword ptr [rsp + 96]
    mov rdx, qword ptr [rsp + 104]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb7:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb8:
    mov rax, qword ptr [rsp + 128]
    mov qword ptr [rsp + 120], rax
    mov rcx, qword ptr [rsp + 120]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb9:
    mov rax, qword ptr [rsp + 144]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb10
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb10:
    mov rcx, qword ptr [rsp + 120]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb11:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 168], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb15
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb12:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb13:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 152]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb14:
    mov rax, qword ptr [rsp + 160]
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb15:
    mov rax, qword ptr [rsp + 168]
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key:
    sub rsp, 168
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb1:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 58
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb2:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb3
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb3:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 96]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_value
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb4:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb5:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 168
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb6:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 88], rax
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb7:
    mov rax, qword ptr [rsp + 120]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb8
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb8:
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb9:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb10:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb11:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 128]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb12:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb13:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb14:
    mov rax, qword ptr [rsp + 152]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value:
    sub rsp, 200
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb0
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 48]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_skip_whitespace
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb1
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb1:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 44
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb2
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb2:
    mov rax, qword ptr [rsp + 80]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb3
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb4
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb3:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 96], rax
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 96]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_string
    mov qword ptr [rsp + 104], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb6
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb4:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 56]
    mov r8, 125
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_has_byte
    mov qword ptr [rsp + 160], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb14
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb5:
    mov rax, qword ptr [rsp + 72]
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 200
    ret
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb6:
    mov rax, qword ptr [rsp + 104]
    mov qword ptr [rsp + 88], rax
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 120], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb7
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb7:
    mov rax, qword ptr [rsp + 120]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb8
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb9
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb8:
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_result_next
    mov qword ptr [rsp + 128], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb11
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb9:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 144], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb13
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb10:
    mov rax, qword ptr [rsp + 112]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb11:
    mov rcx, qword ptr [rsp + 40]
    mov rdx, qword ptr [rsp + 128]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_key
    mov qword ptr [rsp + 136], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb12
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb12:
    mov rax, qword ptr [rsp + 136]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb13:
    mov rax, qword ptr [rsp + 144]
    mov qword ptr [rsp + 112], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb10
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb14:
    mov rax, qword ptr [rsp + 160]
    cmp rax, 0
    jne __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb15
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb16
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb15:
    mov rax, qword ptr [rsp + 56]
    mov rcx, 1
    add rax, rcx
    mov qword ptr [rsp + 168], rax
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_kind_object
    mov qword ptr [rsp + 176], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb18
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb16:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_error_result
    mov qword ptr [rsp + 192], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb20
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb17:
    mov rax, qword ptr [rsp + 152]
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb5
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb18:
    mov rcx, qword ptr [rsp + 168]
    mov rdx, qword ptr [rsp + 176]
    call __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_ok_result
    mov qword ptr [rsp + 184], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb19
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb19:
    mov rax, qword ptr [rsp + 184]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb17
__ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb20:
    mov rax, qword ptr [rsp + 192]
    mov qword ptr [rsp + 152], rax
    jmp __ml_fn___priv_____D__GitHub_Repositories_Mentalogue_inscribe_examples_json_parser_shared_json_mtl_parse_object_after_value.Lbb17
__ml_fn_print_int:
    sub rsp, 176
    lea r10, [rsp + 175]
    lea r11, [rsp + 176]
    mov rax, rcx
    cmp rax, 0
    je __ml_rt_print_int_zero_1
    mov r8, 0
    cmp rax, 0
    jge __ml_rt_print_int_non_negative_2
    neg rax
    mov r8, 1
__ml_rt_print_int_non_negative_2:
__ml_rt_print_int_loop_0:
    mov rcx, 10
    cqo
    idiv rcx
    add rdx, 48
    mov byte ptr [r10], dl
    sub r10, 1
    cmp rax, 0
    jne __ml_rt_print_int_loop_0
    cmp r8, 0
    je __ml_rt_print_int_after_sign_3
    mov rdx, 45
    mov byte ptr [r10], dl
    jmp __ml_rt_print_int_done_4
__ml_rt_print_int_zero_1:
    mov rdx, 48
    mov byte ptr [r10], dl
    jmp __ml_rt_print_int_done_4
__ml_rt_print_int_after_sign_3:
    add r10, 1
__ml_rt_print_int_done_4:
    mov r9, r11
    sub r9, r10
    mov rcx, -11
    call qword ptr [rip + __ml_iat_GetStdHandle]
    mov rcx, rax
    mov rdx, r10
    mov r8, r9
    lea r9, [rsp + 40]
    mov rax, 0
    mov qword ptr [rsp + 32], rax
    call qword ptr [rip + __ml_iat_WriteFile]
    mov rax, 0
    add rsp, 176
    ret
__ml_fn_print_string:
    sub rsp, 176
    mov r10, rcx
    mov r11, rcx
__ml_rt_print_string_loop_5:
    movzx eax, byte ptr [r11]
    cmp rax, 0
    je __ml_rt_print_string_done_6
    add r11, 1
    jmp __ml_rt_print_string_loop_5
__ml_rt_print_string_done_6:
    mov r9, r11
    sub r9, r10
    mov rcx, -11
    call qword ptr [rip + __ml_iat_GetStdHandle]
    mov rcx, rax
    mov rdx, r10
    mov r8, r9
    lea r9, [rsp + 40]
    mov rax, 0
    mov qword ptr [rsp + 32], rax
    call qword ptr [rip + __ml_iat_WriteFile]
    mov rax, 0
    add rsp, 176
    ret
__ml_fn_print_newline:
    sub rsp, 176
    mov r10, OFFSET FLAT:__ml_data_0
    mov r9, 1
    mov rcx, -11
    call qword ptr [rip + __ml_iat_GetStdHandle]
    mov rcx, rax
    mov rdx, r10
    mov r8, r9
    lea r9, [rsp + 40]
    mov rax, 0
    mov qword ptr [rsp + 32], rax
    call qword ptr [rip + __ml_iat_WriteFile]
    mov rax, 0
    add rsp, 176
    ret
__ml_fn_write_line_string:
    sub rsp, 56
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_write_line_string.Lbb0
__ml_fn_write_line_string.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_print_string
    jmp __ml_fn_write_line_string.Lbb1
__ml_fn_write_line_string.Lbb1:
    call __ml_fn_print_newline
    jmp __ml_fn_write_line_string.Lbb2
__ml_fn_write_line_string.Lbb2:
    mov rax, 0
    add rsp, 56
    ret
__ml_fn_write_line_int:
    sub rsp, 56
    mov qword ptr [rsp + 40], rcx
    jmp __ml_fn_write_line_int.Lbb0
__ml_fn_write_line_int.Lbb0:
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_print_int
    jmp __ml_fn_write_line_int.Lbb1
__ml_fn_write_line_int.Lbb1:
    call __ml_fn_print_newline
    jmp __ml_fn_write_line_int.Lbb2
__ml_fn_write_line_int.Lbb2:
    mov rax, 0
    add rsp, 56
    ret
__ml_fn_print_result:
    sub rsp, 104
    mov qword ptr [rsp + 40], rcx
    mov qword ptr [rsp + 48], rdx
    jmp __ml_fn_print_result.Lbb0
__ml_fn_print_result.Lbb0:
    mov rcx, qword ptr [rsp + 48]
    call __ml_fn_parse_document
    mov qword ptr [rsp + 64], rax
    jmp __ml_fn_print_result.Lbb1
__ml_fn_print_result.Lbb1:
    mov rax, qword ptr [rsp + 64]
    mov qword ptr [rsp + 56], rax
    mov rcx, qword ptr [rsp + 40]
    call __ml_fn_write_line_string
    jmp __ml_fn_print_result.Lbb2
__ml_fn_print_result.Lbb2:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn_result_ok
    mov qword ptr [rsp + 72], rax
    jmp __ml_fn_print_result.Lbb3
__ml_fn_print_result.Lbb3:
    mov rcx, qword ptr [rsp + 72]
    call __ml_fn_bool_to_int
    mov qword ptr [rsp + 80], rax
    jmp __ml_fn_print_result.Lbb4
__ml_fn_print_result.Lbb4:
    mov rcx, qword ptr [rsp + 80]
    call __ml_fn_write_line_int
    jmp __ml_fn_print_result.Lbb5
__ml_fn_print_result.Lbb5:
    mov rcx, qword ptr [rsp + 56]
    call __ml_fn_result_kind
    mov qword ptr [rsp + 88], rax
    jmp __ml_fn_print_result.Lbb6
__ml_fn_print_result.Lbb6:
    mov rcx, qword ptr [rsp + 88]
    call __ml_fn_write_line_int
    jmp __ml_fn_print_result.Lbb7
__ml_fn_print_result.Lbb7:
    mov rax, 0
    add rsp, 104
    ret
__ml_fn_main:
    sub rsp, 48
    jmp __ml_fn_main.Lbb0
__ml_fn_main.Lbb0:
    mov rcx, OFFSET FLAT:__ml_data_1
    mov rdx, OFFSET FLAT:__ml_data_2
    call __ml_fn_print_result
    jmp __ml_fn_main.Lbb1
__ml_fn_main.Lbb1:
    mov rcx, OFFSET FLAT:__ml_data_3
    mov rdx, OFFSET FLAT:__ml_data_4
    call __ml_fn_print_result
    jmp __ml_fn_main.Lbb2
__ml_fn_main.Lbb2:
    mov rcx, OFFSET FLAT:__ml_data_5
    mov rdx, OFFSET FLAT:__ml_data_6
    call __ml_fn_print_result
    jmp __ml_fn_main.Lbb3
__ml_fn_main.Lbb3:
    mov rax, 0
    mov qword ptr [rsp + 32], rax
    mov rax, qword ptr [rsp + 32]
    add rsp, 48
    ret
.section .rodata
__ml_data_0:
    .byte 10, 0
__ml_data_1:
    .byte 111, 98, 106, 101, 99, 116, 0
__ml_data_2:
    .byte 123, 92, 34, 110, 97, 109, 101, 92, 34, 58, 92, 34, 65, 100, 97, 92, 34, 44, 92, 34, 97, 99, 116, 105, 118, 101, 92, 34, 58, 116, 114, 117, 101, 44, 92, 34, 115, 99, 111, 114, 101, 115, 92, 34, 58, 91, 49, 44, 50, 44, 51, 93, 125, 0
__ml_data_3:
    .byte 97, 114, 114, 97, 121, 0
__ml_data_4:
    .byte 91, 110, 117, 108, 108, 44, 102, 97, 108, 115, 101, 44, 116, 114, 117, 101, 44, 123, 92, 34, 118, 97, 108, 117, 101, 92, 34, 58, 49, 50, 125, 93, 0
__ml_data_5:
    .byte 105, 110, 118, 97, 108, 105, 100, 0
__ml_data_6:
    .byte 123, 92, 34, 98, 114, 111, 107, 101, 110, 92, 34, 58, 91, 49, 44, 50, 125, 0
__ml_iat_GetStdHandle:
    .quad 0
__ml_iat_WriteFile:
    .quad 0
__ml_iat_ReadFile:
    .quad 0
