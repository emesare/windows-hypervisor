# windows-hypervisor

## ⚠️ WORK IN PROGRESS

Ergonomic and safe bindings to [WHP] (Windows Hypervisor Platform)

## Example

```rs
let mut partition = PartitionBuilder::new()?
        .property(PartitionProperty::ProcessorCount(1))?
        .setup()?;

let mut data = [0xf4; 65536];
data[0xfff0..0xfff0 + 6].copy_from_slice(&[
    0x31, 0xc0, // xor eax,eax
    0x66, 0xb8, 0xef, 0xbe, // mov eax, 0xbeef
]);
partition.map_memory_region(MemoryRegion::from_bytes(
    0xF0000,
    MapGpaRangeFlags::Read | MapGpaRangeFlags::Execute,
    &data,
))?;

let mut vcpu = partition.create_virtual_processor(0x0)?;
let run_exit_ctx = vcpu.run()?;
let beef = match vcpu.get_register(Register::Rax)? {
    RegisterVal::Reg64(v) => v as u16,
    _ => unreachable!(),
};

println!("Ax value: 0x{:x}", beef);
```

[WHP]: https://learn.microsoft.com/en-us/virtualization/api/hypervisor-platform/hypervisor-platform
