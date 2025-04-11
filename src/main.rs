use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(serde_derive::Deserialize, Debug)]
struct InstructionArgument {
    size: u8,
    offset: u8,
}

#[derive(serde_derive::Deserialize, Debug)]
struct Group {
    size: u8,
    mask: u64,
    arguments: HashMap<String, InstructionArgument>,
    #[serde(deserialize_with = "deserialize_invert_hashmap")]
    subgroups: HashMap<u64, String>,
    #[serde(deserialize_with = "deserialize_invert_hashmap")]
    instructions: HashMap<u64, String>,
}

fn deserialize_invert_hashmap<'de, D>(deserializer: D) -> Result<HashMap<u64, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let original_map = HashMap::<String, u64>::deserialize(deserializer)?;
    let inverted_map = original_map
        .iter()
        .map(|(k, v)| (v.clone(), k.clone()))
        .collect::<HashMap<u64, String>>();
    Ok(inverted_map)
}

#[derive(serde_derive::Deserialize, Debug)]
enum Register {
    R8(u8),
    R16(u16),
    R32(u32),
    R64(u64),
}

impl Register {
    fn read(&self) -> u64 {
        match self {
            Register::R8(v) => *v as u64,
            Register::R16(v) => *v as u64,
            Register::R32(v) => *v as u64,
            Register::R64(v) => *v,
        }
    }

    fn write(&mut self, value: u64) {
        match self {
            Register::R8(v) => *v = value as u8,
            Register::R16(v) => *v = value as u16,
            Register::R32(v) => *v = value as u32,
            Register::R64(v) => *v = value,
        }
    }
}

struct ParsedInstruction {
    name: String,
    arguments: HashMap<String, u64>,
}

#[derive(serde_derive::Deserialize, Debug)]
struct Arch {
    groups: HashMap<String, Group>,
    registers: HashMap<String, Register>,
}

type InstructionFn = fn(&mut HashMap<String, Register>, HashMap<String, u64>) -> ();

struct Emulator {
    arch: Arch,
    instructions: HashMap<String, InstructionFn>,
    memory: Box<[u8]>,
    pc: u64,
}

impl Emulator {
    fn new(arch: Arch, instructions: HashMap<String, InstructionFn>, memory: Box<[u8]>) -> Self {
        Self {
            arch,
            instructions,
            memory,
            pc: 0,
        }
    }
    fn parse_group(&mut self, group: &str) -> ParsedInstruction {
        let group = self.arch.groups.get(group).unwrap();
        let group_size = group.size / 8;

        let mut buffer = [0u8; 8];
        self.memory[self.pc as usize..]
            .take(group_size as u64)
            .read(&mut buffer[(8 - group_size) as usize..])
            .unwrap();

        let instruction = u64::from_be_bytes(buffer) & group.mask;

        match group.instructions.get(&instruction) {
            Some(name) => {
                self.pc += group_size as u64;

                let mut parsed_instruction = ParsedInstruction {
                    name: name.clone(),
                    arguments: HashMap::new(),
                };
                for (arg_name, arg) in &group.arguments {
                    let value = (instruction >> arg.offset) & ((1 << arg.size) - 1);
                    parsed_instruction.arguments.insert(arg_name.clone(), value);
                }
                parsed_instruction
            }
            None => {
                let subgroup_name = group.subgroups.get(&instruction).unwrap().clone();
                self.parse_group(&subgroup_name)
            }
        }
    }

    fn emulate(&mut self) {
        let instruction = self.parse_group("main");
        self.instructions.get(&instruction.name).unwrap()(
            &mut self.arch.registers,
            instruction.arguments,
        );
    }
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("chip8.toml")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let arch = toml::from_str::<Arch>(&content).unwrap();

    println!("Arch: {:?}", arch);
    let instructions_map = HashMap::new();

    let mem = [0xA0u8, 0x0, 0x0, 0x0];
    let mut emulator = Emulator::new(arch, instructions_map, mem.into());

    emulator.emulate();

    Ok(())
}
