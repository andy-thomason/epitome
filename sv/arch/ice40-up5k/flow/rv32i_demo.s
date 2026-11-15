# The demonstration program of rv32i.sv, the iCESugar's RV32I computer.
# Assembled by epitome_sv::arch::ice40::rv32i into rv32i_demo.svh; the
# emulator there says what the board should send.
#
# Once a byte arrives (any: it lets the Mac open its serial port first),
# it prints a banner, a signature of a self-test that uses every kind of
# RV32I instruction, and the primes below 200 with the count below 2000
# (a sieve in memory). Then, for each line it receives (up to a CR; LF is
# dropped, and bytes past 200), it sends the line's length in decimal, its
# CRC-32 in hex and the line reversed.
#
# Registers: s11 holds the I/O base throughout; sp the stack, from the
# top of memory. Routines keep s0-s10 (pushing what they use).

        .equ SIEVE_N, 2000

_start: li   sp, 8192
        li   s11, 0x80000000
        li   t0, 1
        sw   t0, 8(s11)             # the LED pin high (the iCESugar's green off)
        call getc
        sw   zero, 8(s11)           # and low (green on): running
        la   a0, banner
        call puts
        call selftest
        call primes
main:   la   a0, prompt
        call puts
        la   a0, line
        li   a1, 200
        call getline
        mv   s0, a0
        call putdec
        li   a0, ' '
        call putc
        la   a0, line
        mv   a1, s0
        call crc32
        call puthex
        li   a0, ' '
        call putc
        la   s1, line
        add  s2, s1, s0
rev:    beq  s2, s1, revd
        addi s2, s2, -1
        lbu  a0, 0(s2)
        call putc
        j    rev
revd:   la   a0, crlf
        call puts
        j    main

# putc(a0): waits while the transmitter is busy, then sends a0's low byte.
putc:   lw   t0, 0(s11)
        andi t0, t0, 1
        bnez t0, putc
        sw   a0, 0(s11)
        ret

# getc() -> a0: waits for a byte.
getc:   lw   t0, 0(s11)
        andi t0, t0, 2
        beqz t0, getc
        lw   a0, 4(s11)
        ret

# puts(a0): a string ending in 0.
puts:   addi sp, sp, -16
        sw   ra, 12(sp)
        sw   s0, 8(sp)
        mv   s0, a0
ps1:    lbu  a0, 0(s0)
        beqz a0, ps2
        call putc
        addi s0, s0, 1
        j    ps1
ps2:    lw   s0, 8(sp)
        lw   ra, 12(sp)
        addi sp, sp, 16
        ret

# getline(a0 = buffer, a1 = most) -> a0 = length: bytes up to a CR.
getline:
        addi sp, sp, -16
        sw   ra, 12(sp)
        sw   s0, 8(sp)
        sw   s1, 4(sp)
        sw   s2, 0(sp)
        mv   s0, a0
        li   s1, 0
        mv   s2, a1
gl1:    call getc
        li   t1, '\r'
        beq  a0, t1, gl2
        li   t1, '\n'
        beq  a0, t1, gl1
        bgeu s1, s2, gl1
        add  t1, s0, s1
        sb   a0, 0(t1)
        addi s1, s1, 1
        j    gl1
gl2:    mv   a0, s1
        lw   s2, 0(sp)
        lw   s1, 4(sp)
        lw   s0, 8(sp)
        lw   ra, 12(sp)
        addi sp, sp, 16
        ret

# putdec(a0): unsigned decimal, by subtracting powers of ten.
putdec: addi sp, sp, -16
        sw   ra, 12(sp)
        sw   s0, 8(sp)
        sw   s1, 4(sp)
        sw   s2, 0(sp)
        mv   s0, a0
        la   s1, pow10
        li   s2, 0                  # a digit has been sent
pd1:    lw   t0, 0(s1)
        beqz t0, pd4
        li   a0, '0'
pd2:    bltu s0, t0, pd3
        sub  s0, s0, t0
        addi a0, a0, 1
        j    pd2
pd3:    addi s1, s1, 4
        li   t1, '0'
        bne  a0, t1, pd5
        bnez s2, pd5
        lw   t2, 0(s1)              # a leading zero, unless the last digit
        bnez t2, pd1
pd5:    li   s2, 1
        call putc
        j    pd1
pd4:    lw   s2, 0(sp)
        lw   s1, 4(sp)
        lw   s0, 8(sp)
        lw   ra, 12(sp)
        addi sp, sp, 16
        ret

# puthex(a0): eight hex digits.
puthex: addi sp, sp, -16
        sw   ra, 12(sp)
        sw   s0, 8(sp)
        sw   s1, 4(sp)
        mv   s0, a0
        li   s1, 28
ph1:    srl  t1, s0, s1
        andi t1, t1, 15
        la   t2, hexdig
        add  t2, t2, t1
        lbu  a0, 0(t2)
        call putc
        addi s1, s1, -4
        bgez s1, ph1
        lw   s1, 4(sp)
        lw   s0, 8(sp)
        lw   ra, 12(sp)
        addi sp, sp, 16
        ret

# crc32(a0 = bytes, a1 = length) -> a0: the CRC-32 (reflected, 0xEDB88320).
crc32:  li   t0, -1
        li   t3, 0xEDB88320
cr1:    beqz a1, cr4
        lbu  t1, 0(a0)
        xor  t0, t0, t1
        li   t2, 8
cr2:    andi t1, t0, 1
        srli t0, t0, 1
        beqz t1, cr3
        xor  t0, t0, t3
cr3:    addi t2, t2, -1
        bnez t2, cr2
        addi a0, a0, 1
        addi a1, a1, -1
        j    cr1
cr4:    not  a0, t0
        ret

# selftest: mixes a value through every kind of instruction, and prints
# "self-test " and the result in hex.
selftest:
        addi sp, sp, -16
        sw   ra, 12(sp)
        sw   s0, 8(sp)
        sw   s1, 4(sp)
        sw   s2, 0(sp)
        la   a0, stmsg
        call puts
        li   s0, 0x12345678
        li   s1, 0
        la   s2, scratch
st1:    slli t0, s0, 5
        srli t1, s0, 27
        or   t0, t0, t1             # rotate left by 5
        xor  s0, t0, s1
        lui  t1, 0x9e378
        addi t1, t1, -0x647         # 0x9e3779b9
        add  s0, s0, t1
        sra  t2, s0, s1             # by s1 mod 32
        srai t3, s0, 7
        sub  t2, t2, t3
        slt  t4, s0, t2
        sltu t5, s0, t2
        slti t6, s0, -5
        sltiu a1, s0, 100
        add  t4, t4, t5
        add  t4, t4, t6
        add  t4, t4, a1
        xor  s0, s0, t4
        and  t5, s0, t2
        andi t6, t2, 0x5a5
        xori t6, t6, -1
        ori  t5, t5, 0x0f0
        add  s0, s0, t5
        sub  s0, s0, t6
        sll  t5, t2, s1
        srl  t6, t2, s1
        xor  s0, s0, t5
        add  s0, s0, t6
        andi t0, s1, 7
        add  t0, t0, s2
        sb   s0, 0(t0)              # bytes, halves and words: stored, read
        andi t1, s1, 3
        slli t1, t1, 1
        add  t1, t1, s2
        sh   s0, 8(t1)
        sw   s0, 16(s2)
        lb   t3, 0(t0)
        lbu  t4, 0(t0)
        lh   t5, 8(t1)
        lhu  t6, 8(t1)
        lw   a1, 16(s2)
        add  t3, t3, t4
        xor  t3, t3, t5
        add  t3, t3, t6
        xor  s0, a1, t3
        lw   t3, 0(s2)
        lw   t4, 4(s2)
        lw   t5, 8(s2)
        lw   t6, 12(s2)
        xor  s0, s0, t3
        add  s0, s0, t4
        xor  s0, s0, t5
        sub  s0, s0, t6
        blt  s0, zero, st2          # branches of each kind
        addi s0, s0, 3
st2:    bge  s0, t2, st3
        xori s0, s0, 0x123
st3:    bltu s0, t2, st4
        addi s0, s0, -77
st4:    bgeu s0, t1, st5
        slli s0, s0, 1
st5:    bne  s0, t2, st6
        addi s0, s0, 1
st6:    beq  s0, t2, st7
        auipc t0, 0x12345           # pc-relative
        la   t1, st6
        sub  t0, t0, t1
        xor  s0, s0, t0
st7:    la   t0, st8
        jalr t1, 4(t0)              # into st8's second instruction
st8:    addi s0, s0, 1000
        xor  s0, s0, t1
        sub  s0, s0, t0             # t1, the link, is st8 itself: t0
        addi s1, s1, 1
        li   t0, 100
        bltu s1, t0, st1
        mv   a0, s0
        call puthex
        la   a0, crlf
        call puts
        lw   s2, 0(sp)
        lw   s1, 4(sp)
        lw   s0, 8(sp)
        lw   ra, 12(sp)
        addi sp, sp, 16
        ret

# primes: a sieve of bytes below SIEVE_N; prints those below 200, then the
# count below SIEVE_N.
primes: addi sp, sp, -16
        sw   ra, 12(sp)
        sw   s0, 8(sp)
        sw   s1, 4(sp)
        sw   s2, 0(sp)
        la   a0, prmsg
        call puts
        la   s0, sieve
        li   s2, SIEVE_N
        li   t0, 0                  # clear: words of zeros
pr0:    add  t1, s0, t0
        sw   zero, 0(t1)
        addi t0, t0, 4
        blt  t0, s2, pr0
        li   s1, 2                  # each p: mark its multiples from 2p
pr1:    add  t0, s0, s1
        lbu  t1, 0(t0)
        bnez t1, pr3
        add  t2, s1, s1
        li   t3, 1
pr2:    bge  t2, s2, pr3
        add  t0, s0, t2
        sb   t3, 0(t0)
        add  t2, t2, s1
        j    pr2
pr3:    addi s1, s1, 1
        blt  s1, s2, pr1
        li   s1, 2                  # print those below 200
pr4:    add  t0, s0, s1
        lbu  t1, 0(t0)
        bnez t1, pr5
        mv   a0, s1
        call putdec
        li   a0, ' '
        call putc
pr5:    addi s1, s1, 1
        li   t0, 200
        blt  s1, t0, pr4
        la   a0, crlf
        call puts
        la   a0, cntmsg
        call puts
        li   s1, 2                  # count them all
        li   a0, 0
pr6:    add  t0, s0, s1
        lbu  t1, 0(t0)
        seqz t1, t1
        add  a0, a0, t1
        addi s1, s1, 1
        blt  s1, s2, pr6
        call putdec
        la   a0, crlf
        call puts
        lw   s2, 0(sp)
        lw   s1, 4(sp)
        lw   s0, 8(sp)
        lw   ra, 12(sp)
        addi sp, sp, 16
        ret

banner: .asciz "\r\nEpitome RV32I on the iCESugar (iCE40UP5K)\r\n"
stmsg:  .asciz "self-test "
prmsg:  .asciz "primes: "
cntmsg: .asciz "primes below 2000: "
prompt: .asciz "> "
crlf:   .asciz "\r\n"
hexdig: .ascii "0123456789abcdef"
        .align 2
pow10:  .word 1000000000, 100000000, 10000000, 1000000, 100000, 10000, 1000, 100, 10, 1, 0
scratch: .space 20
line:   .space 200
        .align 2
sieve:  .space SIEVE_N
