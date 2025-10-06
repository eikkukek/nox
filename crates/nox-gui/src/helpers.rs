use nox::*;

use nox_font::RenderedText;

use crate::*;

pub fn render_text(
    text: &RenderedText,
    render_commands: &mut RenderCommands,
    vertex_buffer: &mut RingBuf,
    index_buffer: &mut RingBuf,
) -> Result<(), Error>
{
    let vertex_buffer_id = vertex_buffer.id();
    let index_buffer_id = index_buffer.id();
    for text in &text.text {
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, text.trigs.vertices.len())?
        };
        let vert_off_mem = unsafe {
            vertex_buffer.allocate(render_commands, text.offsets.len())?
        };
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, text.trigs.indices.len())?
        }; 
        unsafe {
            text.trigs.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), text.trigs.vertices.len());
            text.offsets
                .as_ptr()
                .copy_to_nonoverlapping(vert_off_mem.ptr.as_ptr(), text.offsets.len());
            text.trigs.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), text.trigs.indices.len());
        }
        render_commands.draw_indexed(
            DrawInfo {
                index_count: text.trigs.indices.len() as u32,
                instance_count: text.offsets.len() as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vertex_buffer_id, vert_mem.offset),
                DrawBufferInfo::new(vertex_buffer_id, vert_off_mem.offset),
            ],
            DrawBufferInfo::new(index_buffer_id, idx_mem.offset),
        )?;
    }
    Ok(())
}
