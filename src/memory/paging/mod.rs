use core::ops::{Deref, DerefMut};

use multiboot2::BootInformation;
use x86_64::{structures::paging::PhysFrame, PhysAddr};

use crate::memory::{
    paging::{entry_flags::EntryFlags, mapper::Mapper, temporary_page::TemporaryPage},
    Frame, FrameAllocator,
};

use super::PAGE_SIZE;
pub const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

#[derive(Debug, Clone, Copy)]
pub struct Page {
    number: usize,
}
mod entry;
mod entry_flags;

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xFFFF_8000_0000_0000,
            "invalid address: 0x{address:x}"
        );
        Page {
            number: address / PAGE_SIZE,
        }
    }
    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}
mod mapper;
mod table;
pub struct ActivePageTable {
    mapper: Mapper,
}
impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}
impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}
impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }
    pub fn with<F>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut TemporaryPage,
        f: F,
    ) where
        F: FnOnce(&mut Mapper),
    {
        use x86_64::instructions::tlb;
        use x86_64::registers::control;
        use EntryFlags::{Present, Writeable};

        let backup = {
            let (frame, _flags) = control::Cr3::read();
            let address: u64 = frame.start_address().as_u64();
            Frame::containing_address(address as usize)
        };
        let p4_table = temporary_page.map_table_frame(backup.clone(), self);

        self.p4_mut()[511].set(table.p4_frame.clone(), Present | Writeable);
        tlb::flush_all();
        f(self);

        p4_table[511].set(backup, Present | Writeable);
        tlb::flush_all();
    }
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86_64::registers::control;
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(
                control::Cr3::read().0.start_address().as_u64() as usize
            ),
        };
        unsafe {
            let frame = PhysFrame::containing_address(PhysAddr::new(
                new_table.p4_frame.start_address() as u64,
            ));
            control::Cr3::write(frame, control::Cr3::read().1);
        }
        old_table
    }
}
pub struct InactivePageTable {
    p4_frame: Frame,
}
impl InactivePageTable {
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> InactivePageTable {
        {
            use EntryFlags::{Present, Writeable};
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), Present | Writeable);
        }
        temporary_page.unmap(active_table);
        InactivePageTable { p4_frame: frame }
    }
}
mod temporary_page;
pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation)
where
    A: FrameAllocator,
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe }, allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {

        // … identity map the allocated kernel sections
        let elf_section_tag = boot_info
            .elf_sections_tag()
            .expect("Memory map tag required");
        for section in elf_section_tag.sections() {
            if !section.is_allocated() {
                continue;
            }
            assert!(
                section.start_address() as usize % PAGE_SIZE == 0,
                "Sections need to be page alligned"
            );
            println!(
                "mapping section at addr: {:#x}, size: {:#x?}",
                section.start_address(),
                section.size()
            );
            let flags = EntryFlags::from_elf_section_flags(&section);

            let start_frame = Frame::containing_address(section.start_address() as usize);
            let end_frame = Frame::containing_address(section.end_address() as usize - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }

        }

        // … identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::Writeable, allocator);

        // … identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::Present, allocator);
        }
    });
    let old_table = active_table.switch(new_table);
    println!("NEW TABLE! Hurray");
}
// pub fn test_paging<A>(allocator: &mut A)
// where
//     A: FrameAllocator,
// {
//     let mut page_table = unsafe { ActivePageTable::new() };
//
//     let addr = 42 * 512 * 512 * 4096;
//     let page = Page::containing_address(addr);
//     let frame = allocator.allocate_frame().expect("no more frames");
//     println!(
//         "None = {:?}, map to {:?}",
//         page_table.translate(addr),
//         frame
//     );
//     page_table.map_to(page, frame, 0, allocator);
//     println!("Some = {:?}", page_table.translate(addr));
//     println!("next free frame: {:?}", allocator.allocate_frame());
//
//     println!("{:#x}", unsafe {
//         *(Page::containing_address(addr).start_address() as *const u64)
//     });
//
//     page_table.unmap(Page::containing_address(addr), allocator);
//     println!("None = {:?}", page_table.translate(addr));
//
//     // println!("{:#x}", unsafe {
//     //     *(Page::containing_address(addr).start_address() as *const u64)
//     // });
// }
