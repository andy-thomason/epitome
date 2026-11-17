"""cocotb tests of `fifo.sv`, run on epitome-sv by `run.py`."""

import random

import cocotb
from cocotb.clock import Clock
from cocotb.handle import Force, Release
from cocotb.triggers import (
    ClockCycles,
    FallingEdge,
    NextTimeStep,
    ReadOnly,
    ReadWrite,
    RisingEdge,
    Timer,
    ValueChange,
)


async def reset(dut):
    """Starts the clock and resets the FIFO."""
    cocotb.start_soon(Clock(dut.clk, 10, unit="ns").start())
    dut.rst_n.value = 0
    dut.push.value = 0
    dut.pop.value = 0
    dut.din.value = 0
    await ClockCycles(dut.clk, 2)
    dut.rst_n.value = 1
    await RisingEdge(dut.clk)


async def push(dut, value):
    dut.push.value = 1
    dut.din.value = value
    await RisingEdge(dut.clk)
    dut.push.value = 0


async def pop(dut):
    await FallingEdge(dut.clk)
    value = int(dut.dout.value)
    dut.pop.value = 1
    await RisingEdge(dut.clk)
    dut.pop.value = 0
    return value


@cocotb.test()
async def first_in_first_out(dut):
    """Words come out in the order they went in."""
    await reset(dut)
    assert dut.empty.value == 1
    words = [random.randrange(1 << len(dut.din)) for _ in range(int(dut.DEPTH.value))]
    for w in words:
        await push(dut, w)
    await ReadOnly()
    assert dut.full.value == 1
    assert int(dut.count.value) == len(words)
    got = [await pop(dut) for _ in words]
    assert got == words, (got, words)
    await ReadOnly()
    assert dut.empty.value == 1


@cocotb.test()
async def full_drops_pushes(dut):
    """A push while full is dropped."""
    await reset(dut)
    depth = int(dut.DEPTH.value)
    for i in range(depth + 2):
        await push(dut, i)
    await ReadOnly()
    assert int(dut.count.value) == depth
    assert [int(dut.mem[i].value) for i in range(depth)] == list(range(depth))
    assert all(dut.slot[i].used.value == 1 for i in range(depth))


@cocotb.test()
async def hierarchy_and_values(dut):
    """Parameters, generate blocks, arrays, and values by kind."""
    await reset(dut)
    assert dut.DEPTH.value == 8
    assert dut.WIDTH.value == 16
    assert len(dut.din) == 16
    assert len(dut.mem) == 8
    await push(dut, 0xABCD)
    await ReadOnly()
    assert dut.mem[0].value == 0xABCD
    assert dut.slot[0].used.value == 1
    assert dut.slot[1].used.value == 0
    assert str(dut.dout.value) == format(0xABCD, "016b")


@cocotb.test()
async def triggers(dut):
    """Timers, edges, value changes and the regions of a time slot."""
    await reset(dut)
    start = cocotb.utils.get_sim_time("ns")
    await Timer(3, unit="ns")
    assert cocotb.utils.get_sim_time("ns") == start + 3
    await FallingEdge(dut.clk)
    assert dut.clk.value == 0
    dut.push.value = 1
    dut.din.value = 7
    await ValueChange(dut.count)
    assert int(dut.count.value) == 1
    dut.push.value = 0
    await ReadWrite()
    dut.pop.value = 1
    await ReadOnly()
    assert dut.pop.value == 1
    await NextTimeStep()
    await RisingEdge(dut.clk)
    dut.pop.value = 0
    await ReadOnly()
    assert dut.empty.value == 1


@cocotb.test()
async def force_and_release(dut):
    """A forced output holds until it is released."""
    await reset(dut)
    dut.full.value = Force(1)
    await ReadOnly()
    assert dut.full.value == 1
    await RisingEdge(dut.clk)
    assert dut.full.value == 1
    dut.full.value = Release()
    await ReadOnly()
    assert dut.full.value == 0
