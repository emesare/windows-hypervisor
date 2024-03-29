use windows_hypervisor::{
    flags::MapGpaRangeFlags,
    memory::MemoryRegion,
    partition::{PartitionBuilder, PartitionProperty},
    processor::{Register, RegisterVal},
    query_capability, CapabilityCode,
};

fn main() -> Result<(), windows_hypervisor::Error> {
    for code in [
        CapabilityCode::HypervisorPresent,
        CapabilityCode::Features,
        CapabilityCode::ExtendedVmExits,
        CapabilityCode::ProcessorVendor,
        CapabilityCode::ProcessorFeatures,
        CapabilityCode::ProcessorClFlushSize,
        CapabilityCode::ProcessorXsaveFeatures,
    ] {
        let cap = query_capability(code)?;
        println!("Capability: {:?}", cap);
    }

    let mut partition = PartitionBuilder::new()?
        .property(PartitionProperty::ProcessorCount(1))?
        .setup()?;

    println!("Partition: {:?}", partition);

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

    println!("Virtual Processor: {:?}", vcpu);

    let run_exit_ctx = vcpu.run()?;

    println!("Run Exit Context: {:#?}", run_exit_ctx);

    let beef = match vcpu.get_register(Register::Rax)? {
        RegisterVal::Reg64(v) => v as u16,
        _ => unreachable!(),
    };

    println!("Ax value: 0x{:x}", beef);

    Ok(())
}
