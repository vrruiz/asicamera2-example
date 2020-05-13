extern crate asicamera2;

mod fitswriter;

use std::ffi::CStr;
use std::path::PathBuf;

fn main() {
    println!("Using ZWO ASI C Library via Rust");

    let image_bytes;
    let image_size: usize;

    // Read the number of connected cameras
    let asi_connected_cameras = unsafe { asicamera2::ASIGetNumOfConnectedCameras() };
    println!("Connected cameras: {}", asi_connected_cameras);

    if asi_connected_cameras < 1 {
        return;
    }

    // Get each connected camera's properties into an array
    let mut asi_camera_info = asicamera2::ASI_CAMERA_INFO {
        Name: [0; 64],
        CameraID: 0,
        MaxHeight: 0,
        MaxWidth: 0,
        IsColorCam: asicamera2::ASI_BOOL_ASI_FALSE,
        BayerPattern: asicamera2::ASI_BAYER_PATTERN_ASI_BAYER_RG,
        SupportedBins: [0; 16],
        SupportedVideoFormat: [0; 8],
        PixelSize: 0.0,
        MechanicalShutter: asicamera2::ASI_BOOL_ASI_FALSE,
        ST4Port: asicamera2::ASI_BOOL_ASI_FALSE,
        IsCoolerCam: asicamera2::ASI_BOOL_ASI_FALSE,
        IsUSB3Host: asicamera2::ASI_BOOL_ASI_FALSE,
        IsUSB3Camera: asicamera2::ASI_BOOL_ASI_FALSE,
        ElecPerADU: 0.0,
        BitDepth: 0,
        IsTriggerCam: asicamera2::ASI_BOOL_ASI_FALSE,
        Unused: [0; 16],
    };
    let camera_num = 0;
    let mut success = unsafe { asicamera2::ASIGetCameraProperty(&mut asi_camera_info, camera_num) };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!("Couldn't open ");
    }
    // Print camera's properties
    let name = unsafe {
        CStr::from_ptr(asi_camera_info.Name.as_ptr())
            .to_str()
            .unwrap()
    };
    println!("Camera info");
    println!("  ASI Camera Name: {}", name);
    println!("  Camera ID: {}", asi_camera_info.CameraID);
    println!(
        "  Height and width: {} x {}",
        asi_camera_info.MaxHeight, asi_camera_info.MaxWidth
    );
    println!("  Color: {}", asi_camera_info.IsColorCam);
    println!("  Bayer pattern: {}", asi_camera_info.BayerPattern);
    println!("  Pixel size: {} um", asi_camera_info.PixelSize);
    println!("  e/ADU: {}", asi_camera_info.ElecPerADU);
    println!("  BitDepth: {}", asi_camera_info.BitDepth);

    image_bytes = 2;
    // Calculate image size
    image_size = (asi_camera_info.MaxWidth * asi_camera_info.MaxHeight * image_bytes) as usize;
    println!("Image size: {} bytes", image_size);

    // Open first camera
    println!("Opening camera");
    success = unsafe { asicamera2::ASIOpenCamera(asi_camera_info.CameraID) };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!("Error opening camera #{}", asi_camera_info.CameraID);
        return;
    }

    // Initialize camera
    println!("Initializing camera");
    success = unsafe { asicamera2::ASIInitCamera(asi_camera_info.CameraID) };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!("Error initializing camera #{}", asi_camera_info.CameraID);
        return;
    }

    // Get camera's properties
    let mut asi_num_controls = 0;
    success =
        unsafe { asicamera2::ASIGetNumOfControls(asi_camera_info.CameraID, &mut asi_num_controls) };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!(
            "Error getting number of controls of camera #{}",
            asi_camera_info.CameraID
        );
        return;
    }

    // Get camera mode
    let mut asi_camera_mode = 0;
    success =
        unsafe { asicamera2::ASIGetCameraMode(asi_camera_info.CameraID, &mut asi_camera_mode) };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!(
            "Error getting camera mode of camera #{}",
            asi_camera_info.CameraID
        );
        return;
    }
    println!("Camera mode: {}", asi_camera_mode);

    let mut asi_control_caps: Vec<asicamera2::ASI_CONTROL_CAPS> = Vec::new();
    for i in 0..asi_num_controls {
        let mut asi_control_cap = asicamera2::ASI_CONTROL_CAPS {
            Name: [0; 64],
            Description: [0; 128],
            MaxValue: 0,
            MinValue: 0,
            DefaultValue: 0,
            IsAutoSupported: asicamera2::ASI_BOOL_ASI_FALSE,
            IsWritable: asicamera2::ASI_BOOL_ASI_FALSE,
            ControlType: 0,
            Unused: [0; 32],
        };
        success = unsafe {
            asicamera2::ASIGetControlCaps(asi_camera_info.CameraID, i, &mut asi_control_cap)
        };
        if success == asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
            // Print camera's properties
            let cap_name = unsafe {
                CStr::from_ptr(asi_control_cap.Name.as_ptr())
                    .to_str()
                    .unwrap()
            };
            let cap_description = unsafe {
                CStr::from_ptr(asi_control_cap.Description.as_ptr())
                    .to_str()
                    .unwrap()
            };
            let mut writable = "";
            if asi_control_cap.IsWritable == asicamera2::ASI_BOOL_ASI_TRUE {
                writable = " (set)";
            }
            println!(
                "  Property {}: [{}, {}] = {}{} - {}",
                cap_name,
                asi_control_cap.MinValue,
                asi_control_cap.MaxValue,
                asi_control_cap.DefaultValue,
                writable,
                cap_description
            );
        }
        asi_control_caps.push(asi_control_cap);
    }

    // Set 16 bit mode
    println!("Set RAW16 mode.");
    success = unsafe {
        asicamera2::ASISetROIFormat(
            asi_camera_info.CameraID,
            asi_camera_info.MaxWidth as i32,
            asi_camera_info.MaxHeight as i32,
            1,
            asicamera2::ASI_IMG_TYPE_ASI_IMG_RAW16,
        )
    };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!(
            "Error setting RAW16 mode on camera #{}",
            asi_camera_info.CameraID
        );
        return;
    }
    
    // Exposure
    println!("Start exposure...");
    let mut asi_exp_status = 0;
    success = unsafe { asicamera2::ASIGetExpStatus(asi_camera_info.CameraID, &mut asi_exp_status) };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!(
            "Error getting exposure status of camera #{}",
            asi_camera_info.CameraID
        );
    }
    if asi_exp_status != asicamera2::ASI_EXPOSURE_STATUS_ASI_EXP_IDLE {
        println!("The camera is not idle, cannot start exposure.");
        return;
    }

    // Set exposure time
    let exposure = 10;
    let exposure_time: i64 = exposure * 1000000; // 'exposure' seconds
    println!("Set exposure time: {} seconds", exposure_time / 1000000);
    success = unsafe {
        asicamera2::ASISetControlValue(
            asi_camera_info.CameraID,
            asicamera2::ASI_CONTROL_TYPE_ASI_EXPOSURE as i32,
            exposure_time,
            asicamera2::ASI_BOOL_ASI_FALSE as i32,
        )
    };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!("Cannot set exposure time.");
        return;
    }

    // Start exposure
    success = unsafe {
        asicamera2::ASIStartExposure(
            asi_camera_info.CameraID,
            asicamera2::ASI_BOOL_ASI_FALSE as i32,
        )
    };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!("Cannot start exposure.");
        return;
    }
    #[allow(unused_assignments)]
    let mut captured = 0;
    // Wait until the exposure has been made
    loop {
        unsafe { asicamera2::ASIGetExpStatus(asi_camera_info.CameraID, &mut asi_exp_status) };
        if asi_exp_status == asicamera2::ASI_EXPOSURE_STATUS_ASI_EXP_SUCCESS {
            println!("Successful exposure");
            captured = 1;
            break;
        } else if asi_exp_status == asicamera2::ASI_EXPOSURE_STATUS_ASI_EXP_FAILED {
            println!("Failed exposure");
            return;
        }
    }

    if captured != 1 {
        println!("Cannot happen");
        return;
    }
    // Get exposure data
    println!("Image size: {}", image_size);
    println!("Image bytes: {}", image_bytes);
    let mut asi_image: Vec<u8> = Vec::with_capacity(image_size as usize);
    asi_image.resize(image_size, 0);
    success = unsafe {
        asicamera2::ASIGetDataAfterExp(
            asi_camera_info.CameraID,
            asi_image.as_mut_ptr(),
            image_size as i64,
        )
    };
    if success != asicamera2::ASI_ERROR_CODE_ASI_SUCCESS as i32 {
        println!("Couldn't read exposure data");
        return;
    }
    // Print first bytes of the acquired data
    println!("Image data:");
    for i in 0..20 {
        print!("{} ", asi_image[i]);
    }

    // Write FITS image
    if image_bytes == 2 {
        let fits_hd = fitswriter::FitsHeaderData {
            bitpix: 8 * image_bytes as i64,
            naxis: 2u64,
            naxis_vec: vec![
                asi_camera_info.MaxWidth as u64,
                asi_camera_info.MaxHeight as u64,
            ],
            bzero: 32767,
            bscale: 1,
            datamin: 0,
            datamax: 0,
            history: vec![String::new()],
            comment: vec![String::new()],
            data_bytes: asi_image,
        };
        let fits_path = PathBuf::from("/home/rvr/asicamera2-test.fits");
        let result = fitswriter::fits_write_data(&fits_path, &fits_hd);
        match result {
            Ok(_m) => {
                println!("Image saved");
            }
            Err(e) => {
                println!("Error saving image: {}", e);
            }
        }
    }
}
