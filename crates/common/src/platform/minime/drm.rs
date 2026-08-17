use anyhow::{Context, Result, anyhow};
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::ptr;
use tiny_skia::{Pixmap, PixmapMut, PixmapRef};

use crate::display::Display;
use crate::display::color::Color;
use crate::geom::Rect;
use crate::platform::minime::traits::Traits;

#[repr(C)]
struct DrmModeCreateDumb {
    height: u32,
    width: u32,
    bpp: u32,
    flags: u32,
    handle: u32,
    pitch: u32,
    size: u64,
}

#[repr(C)]
struct DrmModeMapDumb {
    handle: u32,
    pad: u32,
    offset: u64,
}

#[repr(C)]
struct DrmModeDestroyDumb {
    handle: u32,
}

#[repr(C)]
struct DrmModeFbCmd {
    fb_id: u32,
    width: u32,
    height: u32,
    pitch: u32,
    bpp: u32,
    depth: u32,
    handle: u32,
}

#[repr(C)]
struct DrmModeCardRes {
    fb_id_ptr: u64,
    crtc_id_ptr: u64,
    connector_id_ptr: u64,
    encoder_id_ptr: u64,
    count_fbs: u32,
    count_crtcs: u32,
    count_connectors: u32,
    count_encoders: u32,
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct DrmModeModeinfo {
    clock: u32,
    hdisplay: u16,
    hsync_start: u16,
    hsync_end: u16,
    htotal: u16,
    hskew: u16,
    vdisplay: u16,
    vsync_start: u16,
    vsync_end: u16,
    vtotal: u16,
    vscan: u16,
    vrefresh: u32,
    flags: u32,
    type_: u32,
    name: [u8; 32],
}

#[repr(C)]
struct DrmModeGetConnector {
    encoders_ptr: u64,
    modes_ptr: u64,
    props_ptr: u64,
    prop_values_ptr: u64,
    count_modes: u32,
    count_props: u32,
    count_encoders: u32,
    encoder_id: u32,
    connector_id: u32,
    connector_type: u32,
    connector_type_id: u32,
    connection: u32,
    mm_width: u32,
    mm_height: u32,
    subpixel: u32,
    pad: u32,
}

#[repr(C)]
struct DrmModeGetEncoder {
    encoder_id: u32,
    encoder_type: u32,
    crtc_id: u32,
    possible_crtcs: u32,
    possible_clones: u32,
}

#[repr(C)]
struct DrmModeCrtc {
    set_connectors_ptr: u64,
    count_connectors: u32,
    crtc_id: u32,
    fb_id: u32,
    x: u32,
    y: u32,
    gamma_size: u32,
    mode_valid: u32,
    mode: DrmModeModeinfo,
}

const DRM_IOCTL_BASE: u8 = b'd';
const DRM_COMMAND_BASE: u8 = 0xA0;

nix::ioctl_readwrite!(
    drm_get_resources,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x00,
    DrmModeCardRes
);
nix::ioctl_readwrite!(
    drm_set_crtc,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x02,
    DrmModeCrtc
);
nix::ioctl_readwrite!(
    drm_get_encoder,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x06,
    DrmModeGetEncoder
);
nix::ioctl_readwrite!(
    drm_get_connector,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x07,
    DrmModeGetConnector
);
nix::ioctl_readwrite!(
    drm_add_fb,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x0E,
    DrmModeFbCmd
);
nix::ioctl_readwrite!(drm_rm_fb, DRM_IOCTL_BASE, DRM_COMMAND_BASE + 0x0F, u32);
nix::ioctl_readwrite!(
    drm_create_dumb,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x12,
    DrmModeCreateDumb
);
nix::ioctl_readwrite!(
    drm_map_dumb,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x13,
    DrmModeMapDumb
);
nix::ioctl_readwrite!(
    drm_destroy_dumb,
    DRM_IOCTL_BASE,
    DRM_COMMAND_BASE + 0x14,
    DrmModeDestroyDumb
);

pub struct MinimeDrmDisplay {
    pixmap: Pixmap,
    file: File,
    map_ptr: *mut u8,
    map_size: usize,
    pitch: usize,
    handle: u32,
    fb_id: u32,
    saved: Vec<Pixmap>,
}

unsafe impl Send for MinimeDrmDisplay {}
unsafe impl Sync for MinimeDrmDisplay {}

impl MinimeDrmDisplay {
    pub fn new(traits: &Traits) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/dri/card0")
            .context("failed to open /dev/dri/card0")?;
        let fd = file.as_raw_fd();

        let mut res = DrmModeCardRes {
            fb_id_ptr: 0,
            crtc_id_ptr: 0,
            connector_id_ptr: 0,
            encoder_id_ptr: 0,
            count_fbs: 0,
            count_crtcs: 0,
            count_connectors: 0,
            count_encoders: 0,
            min_width: 0,
            max_width: 0,
            min_height: 0,
            max_height: 0,
        };
        unsafe { drm_get_resources(fd, &mut res) }.context("DRM_IOCTL_MODE_GETRESOURCES failed")?;

        if res.count_connectors == 0 || res.count_crtcs == 0 {
            return Err(anyhow!("no DRM connectors or CRTCs found"));
        }

        let mut crtcs = vec![0u32; res.count_crtcs as usize];
        let mut conns = vec![0u32; res.count_connectors as usize];
        let mut encs = vec![0u32; res.count_encoders as usize];
        let mut fbs = vec![0u32; res.count_fbs as usize];

        res.crtc_id_ptr = crtcs.as_mut_ptr() as u64;
        res.connector_id_ptr = conns.as_mut_ptr() as u64;
        res.encoder_id_ptr = encs.as_mut_ptr() as u64;
        res.fb_id_ptr = fbs.as_mut_ptr() as u64;
        unsafe { drm_get_resources(fd, &mut res) }
            .context("DRM_IOCTL_MODE_GETRESOURCES second call failed")?;

        let mut chosen_conn = 0u32;
        let mut chosen_crtc = 0u32;
        let mut chosen_mode = DrmModeModeinfo::default();
        let mut found = false;

        for &conn_id in &conns {
            let mut conn_req = DrmModeGetConnector {
                encoders_ptr: 0,
                modes_ptr: 0,
                props_ptr: 0,
                prop_values_ptr: 0,
                count_modes: 0,
                count_props: 0,
                count_encoders: 0,
                encoder_id: 0,
                connector_id: conn_id,
                connector_type: 0,
                connector_type_id: 0,
                connection: 0,
                mm_width: 0,
                mm_height: 0,
                subpixel: 0,
                pad: 0,
            };
            if unsafe { drm_get_connector(fd, &mut conn_req) }.is_err() {
                continue;
            }
            if conn_req.connection != 1 || conn_req.count_modes == 0 {
                continue;
            }

            let mut modes = vec![DrmModeModeinfo::default(); conn_req.count_modes as usize];
            let mut enc_ids = vec![0u32; conn_req.count_encoders as usize];
            conn_req.modes_ptr = modes.as_mut_ptr() as u64;
            conn_req.encoders_ptr = enc_ids.as_mut_ptr() as u64;

            if unsafe { drm_get_connector(fd, &mut conn_req) }.is_ok() {
                chosen_conn = conn_id;
                chosen_mode = modes[0];

                if conn_req.encoder_id != 0 {
                    let mut enc_req = DrmModeGetEncoder {
                        encoder_id: conn_req.encoder_id,
                        encoder_type: 0,
                        crtc_id: 0,
                        possible_crtcs: 0,
                        possible_clones: 0,
                    };
                    if unsafe { drm_get_encoder(fd, &mut enc_req) }.is_ok() && enc_req.crtc_id != 0
                    {
                        chosen_crtc = enc_req.crtc_id;
                    }
                }
                if chosen_crtc == 0 && !crtcs.is_empty() {
                    chosen_crtc = crtcs[0];
                }
                found = true;
                break;
            }
        }

        if !found || chosen_crtc == 0 {
            return Err(anyhow!("failed to find active DRM connector/CRTC"));
        }

        let width = traits.screen_width;
        let height = traits.screen_height;

        let mut cd = DrmModeCreateDumb {
            width,
            height,
            bpp: 32,
            flags: 0,
            handle: 0,
            pitch: 0,
            size: 0,
        };
        unsafe { drm_create_dumb(fd, &mut cd) }.context("DRM_IOCTL_MODE_CREATE_DUMB failed")?;

        let mut fb_cmd = DrmModeFbCmd {
            fb_id: 0,
            width,
            height,
            pitch: cd.pitch,
            bpp: 32,
            depth: 24,
            handle: cd.handle,
        };
        if let Err(e) = unsafe { drm_add_fb(fd, &mut fb_cmd) } {
            let mut dd = DrmModeDestroyDumb { handle: cd.handle };
            let _ = unsafe { drm_destroy_dumb(fd, &mut dd) };
            return Err(anyhow!("DRM_IOCTL_MODE_ADDFB failed: {e}"));
        }

        let mut md = DrmModeMapDumb {
            handle: cd.handle,
            pad: 0,
            offset: 0,
        };
        if let Err(e) = unsafe { drm_map_dumb(fd, &mut md) } {
            let mut dd = DrmModeDestroyDumb { handle: cd.handle };
            let _ = unsafe { drm_destroy_dumb(fd, &mut dd) };
            let _ = unsafe { drm_rm_fb(fd, &mut fb_cmd.fb_id) };
            return Err(anyhow!("DRM_IOCTL_MODE_MAP_DUMB failed: {e}"));
        }

        let map_ptr = unsafe {
            nix::libc::mmap(
                ptr::null_mut(),
                cd.size as usize,
                nix::libc::PROT_READ | nix::libc::PROT_WRITE,
                nix::libc::MAP_SHARED,
                fd,
                md.offset as nix::libc::off_t,
            )
        };
        if map_ptr == nix::libc::MAP_FAILED {
            let mut dd = DrmModeDestroyDumb { handle: cd.handle };
            let _ = unsafe { drm_destroy_dumb(fd, &mut dd) };
            let _ = unsafe { drm_rm_fb(fd, &mut fb_cmd.fb_id) };
            return Err(anyhow!("DRM dumb buffer mmap failed"));
        }

        let mut conn_ids = [chosen_conn];
        let mut crtc_req = DrmModeCrtc {
            set_connectors_ptr: conn_ids.as_mut_ptr() as u64,
            count_connectors: 1,
            crtc_id: chosen_crtc,
            fb_id: fb_cmd.fb_id,
            x: 0,
            y: 0,
            gamma_size: 0,
            mode_valid: 1,
            mode: chosen_mode,
        };
        let _ = unsafe { drm_set_crtc(fd, &mut crtc_req) };

        let pixmap =
            Pixmap::new(width, height).ok_or_else(|| anyhow!("failed to create display pixmap"))?;

        Ok(Self {
            pixmap,
            file,
            map_ptr: map_ptr as *mut u8,
            map_size: cd.size as usize,
            pitch: cd.pitch as usize,
            handle: cd.handle,
            fb_id: fb_cmd.fb_id,
            saved: Vec::new(),
        })
    }
}

impl Drop for MinimeDrmDisplay {
    fn drop(&mut self) {
        let fd = self.file.as_raw_fd();
        if !self.map_ptr.is_null() && self.map_ptr != nix::libc::MAP_FAILED as *mut u8 {
            unsafe {
                ptr::write_bytes(self.map_ptr, 0, self.map_size);
                nix::libc::munmap(self.map_ptr as *mut nix::libc::c_void, self.map_size);
            }
        }
        if self.fb_id != 0 {
            let mut fb_id = self.fb_id;
            let _ = unsafe { drm_rm_fb(fd, &mut fb_id) };
        }
        if self.handle != 0 {
            let mut dd = DrmModeDestroyDumb {
                handle: self.handle,
            };
            let _ = unsafe { drm_destroy_dumb(fd, &mut dd) };
        }
    }
}

impl Display for MinimeDrmDisplay {
    fn width(&self) -> u32 {
        self.pixmap.width()
    }

    fn height(&self) -> u32 {
        self.pixmap.height()
    }

    fn pixmap(&self) -> PixmapRef<'_> {
        self.pixmap.as_ref()
    }

    fn pixmap_mut(&mut self) -> PixmapMut<'_> {
        self.pixmap.as_mut()
    }

    fn map_pixels<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(Color) -> Color,
    {
        for pixel in self.pixmap.pixels_mut() {
            *pixel = f((*pixel).into()).into();
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let src_data = self.pixmap.data();

        // 0° direct blit into mapped DRM dumb buffer
        if self.pitch == width * 4 {
            unsafe {
                ptr::copy_nonoverlapping(src_data.as_ptr(), self.map_ptr, width * height * 4);
            }
        } else {
            for y in 0..height {
                let src_row = &src_data[y * width * 4..(y + 1) * width * 4];
                unsafe {
                    let dst_row = self.map_ptr.add(y * self.pitch);
                    ptr::copy_nonoverlapping(src_row.as_ptr(), dst_row, width * 4);
                }
            }
        }
        Ok(())
    }

    fn save(&mut self) -> Result<()> {
        self.saved.push(self.pixmap.clone());
        Ok(())
    }

    fn load(&mut self, rect: Rect) -> Result<()> {
        let saved = self.saved.last().context("no saved image")?;
        for y in rect.y.max(0) as usize..(rect.y.max(0) as u32 + rect.h).min(self.height()) as usize
        {
            for x in
                rect.x.max(0) as usize..(rect.x.max(0) as u32 + rect.w).min(self.width()) as usize
            {
                let index = y * self.width() as usize + x;
                self.pixmap.pixels_mut()[index] = saved.pixels()[index];
            }
        }
        Ok(())
    }

    fn pop(&mut self) -> bool {
        self.saved.pop();
        !self.saved.is_empty()
    }
}
