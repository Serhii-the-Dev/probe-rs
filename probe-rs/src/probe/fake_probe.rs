use std::fmt::Debug;

use crate::{
    architecture::arm::{
        ap::{memory_ap::mock::MockMemoryAp, AccessPort, MemoryAp},
        communication_interface::{SwdSequence, UninitializedArmProbe},
        dp::DebugPortVersion,
        memory::adi_v5_memory_interface::ADIMemoryInterface,
        ArmProbeInterface, DapAccess, MemoryApInformation, PortType, RawDapAccess, SwoAccess,
    },
    DebugProbe, DebugProbeError, DebugProbeSelector, Error, Memory, Probe, WireProtocol,
};

pub struct FakeProbe {
    protocol: WireProtocol,
    speed: u32,

    dap_register_read_handler:
        Option<Box<dyn Fn(PortType, u8) -> Result<u32, DebugProbeError> + Send>>,

    dap_register_write_handler:
        Option<Box<dyn Fn(PortType, u8, u32) -> Result<(), DebugProbeError> + Send>>,
}

impl Debug for FakeProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeProbe")
            .field("protocol", &self.protocol)
            .field("speed", &self.speed)
            .finish()
    }
}

impl FakeProbe {
    pub fn new() -> Self {
        FakeProbe {
            protocol: WireProtocol::Swd,
            speed: 1000,

            dap_register_read_handler: None,
            dap_register_write_handler: None,
        }
    }

    pub fn handle_dap_register_read(
        &mut self,
        handler: Box<dyn Fn(PortType, u8) -> Result<u32, DebugProbeError> + Send>,
    ) {
        self.dap_register_read_handler = Some(handler);
    }

    pub fn handle_dap_register_write(
        &mut self,
        handler: Box<dyn Fn(PortType, u8, u32) -> Result<(), DebugProbeError> + Send>,
    ) {
        self.dap_register_write_handler = Some(handler);
    }

    pub fn into_probe(self) -> Probe {
        Probe::from_specific_probe(Box::new(self))
    }
}

impl Default for FakeProbe {
    fn default() -> Self {
        FakeProbe::new()
    }
}

impl DebugProbe for FakeProbe {
    fn new_from_selector(
        _selector: impl Into<DebugProbeSelector>,
    ) -> Result<Box<Self>, DebugProbeError>
    where
        Self: Sized,
    {
        Ok(Box::new(FakeProbe::new()))
    }

    /// Get human readable name for the probe
    fn get_name(&self) -> &str {
        "Mock probe for testing"
    }

    fn speed(&self) -> u32 {
        self.speed
    }

    fn set_speed(&mut self, speed_khz: u32) -> Result<u32, DebugProbeError> {
        self.speed = speed_khz;

        Ok(speed_khz)
    }

    fn attach(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn select_protocol(&mut self, protocol: WireProtocol) -> Result<(), DebugProbeError> {
        self.protocol = protocol;

        Ok(())
    }

    /// Leave debug mode
    fn detach(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    /// Resets the target device.
    fn target_reset(&mut self) -> Result<(), DebugProbeError> {
        Err(DebugProbeError::CommandNotSupportedByProbe)
    }

    fn target_reset_assert(&mut self) -> Result<(), DebugProbeError> {
        unimplemented!()
    }

    fn target_reset_deassert(&mut self) -> Result<(), DebugProbeError> {
        unimplemented!()
    }

    fn into_probe(self: Box<Self>) -> Box<dyn DebugProbe> {
        self
    }

    fn try_get_arm_interface<'probe>(
        self: Box<Self>,
    ) -> Result<Box<dyn UninitializedArmProbe + 'probe>, (Box<dyn DebugProbe>, DebugProbeError)>
    {
        // Ok(Box::new(FakeArmInterface::new(self)))
        todo!();
    }

    fn has_arm_interface(&self) -> bool {
        true
    }
}

impl RawDapAccess for FakeProbe {
    /// Reads the DAP register on the specified port and address
    fn raw_read_register(&mut self, port: PortType, addr: u8) -> Result<u32, DebugProbeError> {
        if let Some(handler) = &self.dap_register_read_handler {
            handler(port, addr)
        } else {
            Err(DebugProbeError::CommandNotSupportedByProbe)
        }
    }

    /// Writes a value to the DAP register on the specified port and address
    fn raw_write_register(
        &mut self,
        port: PortType,
        addr: u8,
        value: u32,
    ) -> Result<(), DebugProbeError> {
        if let Some(handler) = &self.dap_register_write_handler {
            handler(port, addr, value)
        } else {
            Err(DebugProbeError::CommandNotSupportedByProbe)
        }
    }

    fn swj_sequence(&mut self, bit_len: u8, bits: u64) -> Result<(), DebugProbeError> {
        todo!()
    }

    fn swj_pins(
        &mut self,
        pin_out: u32,
        pin_select: u32,
        pin_wait: u32,
    ) -> Result<u32, DebugProbeError> {
        todo!()
    }

    fn into_probe(self: Box<Self>) -> Box<dyn DebugProbe> {
        todo!()
    }
}

#[derive(Debug)]
struct FakeArmInterface {
    probe: Box<FakeProbe>,

    memory_ap: MockMemoryAp,
}

impl FakeArmInterface {
    fn new(probe: Box<FakeProbe>) -> Self {
        let memory_ap = MockMemoryAp::with_pattern();
        FakeArmInterface { probe, memory_ap }
    }
}

impl ArmProbeInterface for FakeArmInterface {
    fn memory_interface(&mut self, access_port: MemoryAp) -> Result<Memory<'_>, Error> {
        let ap_information = MemoryApInformation {
            port_number: access_port.port_number(),
            only_32bit_data_size: false,
            debug_base_address: 0xf000_0000,
            supports_hnonsec: false,
        };

        todo!("Fix this!");

        // let memory = ADIMemoryInterface::new(&mut self.memory_ap, &ap_information)?;

        // Ok(Memory::new(memory, access_port))
    }

    fn ap_information(
        &self,
        _access_port: crate::architecture::arm::ap::GenericAp,
    ) -> Option<&crate::architecture::arm::ApInformation> {
        todo!()
    }

    fn num_access_ports(&self) -> usize {
        1
    }

    fn read_from_rom_table(
        &mut self,
    ) -> Result<Option<crate::architecture::arm::ArmChipInfo>, Error> {
        Ok(None)
    }

    fn close(self: Box<Self>) -> Probe {
        Probe::from_attached_probe(self.probe)
    }
}

impl SwoAccess for FakeArmInterface {
    fn enable_swo(&mut self, _config: &crate::architecture::arm::SwoConfig) -> Result<(), Error> {
        unimplemented!()
    }

    fn disable_swo(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn read_swo_timeout(&mut self, _timeout: std::time::Duration) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }
}

impl SwdSequence for FakeArmInterface {
    fn swj_sequence(&mut self, _bit_len: u8, _bits: u64) -> Result<(), Error> {
        todo!()
    }
    fn swj_pins(&mut self, _pin_out: u32, _pin_select: u32, _pin_wait: u32) -> Result<u32, Error> {
        todo!()
    }
}

impl DapAccess for FakeArmInterface {
    fn debug_port_version(&self) -> DebugPortVersion {
        DebugPortVersion::DPv2
    }
    fn read_raw_dp_register(&mut self, _address: u8) -> Result<u32, DebugProbeError> {
        todo!()
    }
    fn write_raw_dp_register(&mut self, _address: u8, _value: u32) -> Result<(), DebugProbeError> {
        todo!()
    }
    fn read_raw_ap_register(
        &mut self,
        _port_number: u8,
        _address: u8,
    ) -> Result<u32, DebugProbeError> {
        todo!()
    }
    fn read_raw_ap_register_repeated(
        &mut self,
        _port: u8,
        _address: u8,
        _values: &mut [u32],
    ) -> Result<(), DebugProbeError> {
        todo!()
    }
    fn write_raw_ap_register(
        &mut self,
        _port: u8,
        _address: u8,
        _value: u32,
    ) -> Result<(), DebugProbeError> {
        todo!()
    }
    fn write_raw_ap_register_repeated(
        &mut self,
        _port: u8,
        _address: u8,
        _values: &[u32],
    ) -> Result<(), DebugProbeError> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::FakeProbe;

    #[test]
    fn create_session_with_fake_probe() {
        let mut fake_probe = FakeProbe::new();

        let probe = fake_probe.into_probe();

        probe.attach("nrf51822").unwrap();
    }
}
