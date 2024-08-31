/* **********************************************************************************************************************
* This program will read an Arduino serial stream that decodes a minx remote controler                                  *
* the Arduino uses a standard IR reciever (KY-022) connected to pin 3                                                   *
* The Arudino code is stored at the bootom of the program - it does use the IRremote library by shirriff V4.4.0         *
*                                                                                                                       *
* Developed by sjf with lots of help from the internet Aug 2024                                                         *
*                                                                                                                       *
*********************************************************************************************************************** */

/* Sample o/p from running program
Program to turn RC5 IR codes into Address and Command info (with toggle) from 13 bit data
This is the data the Cambrideg Audio Minx remote o/p's

 Remote Control command is: "MP3" Toggle is: 0
 Remote Control command is: "MP3" Toggle is: 0
 Remote Control command is: "A1" Toggle is: 1
 Remote Control command is: "A2" Toggle is: 0
 Remote Control command is: "Bluetooth" Toggle is: 1
 Remote Control command is: "Bluetooth" Toggle is: 1
 Remote Control command is: "D1" Toggle is: 0
 Remote Control command is: "D2" Toggle is: 1

*/

//don't for get to add serial = "0.4.0" to cargo.toml

use std::collections::HashMap;

use serial::prelude::*;
use std::io::prelude::*;
use std::time::Duration;
use std::io::BufReader;

const DEBUG: bool = false;


fn main() -> Result<(), serial::Error> {

    let rc5_sc17 = HashMap::from([
        (64 as u32, "Volume Up"),
        (65 as u32, "Volume Down"),
        (67 as u32, "Mute Toggle"),
        (111 as u32, "Internet Radio"),
        (114 as u32, "Services"),
        (112 as u32, "Media"),
        (115 as u32, "Podcasts"),
        (123 as u32, "Tone//Balance"),
    ]);

    let rc5_sc27 = HashMap::from([
        (88 as u32, "Memory"),
        (2 as u32, "Standby Toggle"),
        (102 as u32, "Power On"),
        (103 as u32, "Power Off"),
        (51 as u32, "Brightness"),
        (9 as u32, "Reply"),
        (32 as u32, "MP3"),
        (33 as u32, "A1"),
        (34 as u32, "A2"),
        (31 as u32, "Bluetooth"),
        (35 as u32, "D1"),
        (36 as u32, "D2"),
        (24 as u32, "Play/Pause"),
        (19, "Down"),
        (27, "Stop/Cancel"),
        (25, "Skip Forward"),
        (18, "Right"),
        (17, "Select"),
        (16, "Left"),
        (23, "Skip Back"),
        (22, "Return"),
        (13, "up"),
        (12, "Home"),
        (126, "Mute On"),
        (127, "Mute Off"),
        (28, "Info"),                
        (21, "Repeat"),
        (20, "Random"),

    ]);

    println!("Program to turn RC5 IR codes into Address and Command info (with toggle) from 13 bit data");
    println!("This is the data the Cambrideg Audio Minx remote o/p's");
    println!("");

    //now read from serial port 
    let mut port = serial::open("/dev/ttyACM0").unwrap();
    //interact(&mut port).unwrap();
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
        })?;
        
    port.set_timeout(Duration::from_millis(1000))?;
        
    let reader = BufReader::new(port);
    //let mut lc = 1;
    for line in reader.lines() {
        if line.is_ok() {
            //println!("Line: {}, Info: {:?}", lc, line.unwrap_or("Reading failed".into()));
            let s1 = line.unwrap_or("Reading failed".into());
            if DEBUG { println!("String we're woking on is: {s1}")};
            let addr: u32;
            let cmd: u32;
            let raw: u32;  
            
            if &s1 != "" && (&s1[0..4] == "P=17".to_string()) {
                //only work on line we're interetsed in
                if DEBUG { println!("Tokenising String: {s1}") };

                let tokens: Vec<_> = s1.split_whitespace().map(str::to_string).collect();
                if DEBUG { 
                    for token in &tokens {
                        println!("Tokens are: {}", token);
                    }
                }

                //we know that for rc5 the order is P, A, C, Raw so
                addr = take_token_and_return_value(&tokens[1]);
                cmd = take_token_and_return_value(&tokens[2]);
                raw = take_token_and_return_value(&tokens[3]);

                if DEBUG { println!("Addr : {:#04X}, cmd: {:#04X}, raw: {:#04X}", addr, cmd, raw)};
                //decode_rc5_addr_cmd(addr, cmd, rc5_sc17.clone(), rc5_sc27.clone());
                decode_rc5_raw_data(raw, rc5_sc17.clone(), rc5_sc27.clone());
            }
        }
    }
    Ok(())
}

fn take_token_and_return_value(token: &str) -> u32 {
    let parts: Vec<String> = token.split("=").map(str::to_string).collect();
    if DEBUG { println!("fn: take_token_and_return_value token split gives:  Part 1 is {}, part2 is: {}", parts[0], parts[1])};
    //now convert to u32 
    let without_prefix = parts[1].trim_start_matches("0x");
    let z = u32::from_str_radix(without_prefix, 16);
    return z.unwrap()
}

fn decode_rc5_raw_data (rc5_rcd_data: u32, rc5_sc17: HashMap<u32, &str>, rc5_sc27: HashMap<u32, &str>) {
    //Decodes raw hex from a decoded remote control feed
    let rc5_1: u32 = rc5_rcd_data;               //represents a RCx7 command 
    let rc5_cmd: u32 = 0b0000000000111111;     //mask for lowest 6 bits to give command   0x3F  0b0000000000111111
    let rc5_addr: u32 = 0b0000011111000000;    //mask for address bits 7-11               0x7C0 0b0000011111000000
    let rc5_tog: u32 = 0b0000100000000000;     //mask for toggle bit 12                   0x800 0b0000100000000000
    let mut cmd: u32 = rc5_1 & rc5_cmd;           //this should give the lowest 6 bits
    let addr: u32 = (rc5_1 & rc5_addr)>>6;    //this should give the address but shifted 6 bits to high 
    let tog: u32 = (rc5_1 & rc5_tog)>>11;     //

    //in the protocol there seems to be a rc5x where an additional +0x40 is added to the command (this seems to align with the Minx sys code 17)
    //since cannot get to raw data from bit stream adding this a work round
    //info from https://github.com/Arduino-IRremote/Arduino-IRremote/blob/master/src/ir_RC5_RC6.hpp

    if addr == 0x11 { cmd += 0x40 }; //address 17 in Dec

    if DEBUG { 
        println!("\nData to break into RC5 code is: {:#04X}", rc5_1);
        println!("Dec Address is: {}, Command is: {}, Toggle is: {}", addr, cmd, tog);
        println!("Hex Address is: {:#02X}, Command is: {:#02X}, Toggle is: {}", addr, cmd, tog);
    }

    if addr == 0x11 && rc5_sc17.contains_key(&cmd) {
        println!(" Remote Control command is: {:?} Toggle is: {}", rc5_sc17.get(&cmd).unwrap(), tog);
    }
    else if addr == 0x1B && rc5_sc27.contains_key(&cmd) {
        println!(" Remote Control command is: {:?} Toggle is: {}", rc5_sc27.get(&cmd).unwrap(), tog);
    }
    else { println!("Command code not recognised....") };
}


/* *************************************************************************************************************
*  ------------------------------------------------ ARDUINO CODE -----------------------------------------------
/*
 * SimpleReceiver.cpp
 *
 * Demonstrates receiving ONLY NEC protocol IR codes with IRremote
 * If no protocol is defined, all protocols (except Bang&Olufsen) are active.
 *
 *  This file is part of Arduino-IRremote https://github.com/Arduino-IRremote/Arduino-IRremote.
 *
 ************************************************************************************
 * MIT License
 *
 * Copyright (c) 2020-2023 Armin Joachimsmeyer
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is furnished
 * to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
 * INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
 * PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF
 * CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
 * OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 ************************************************************************************
 */

#include <Arduino.h>

/*
 * Specify which protocol(s) should be used for decoding.
 * If no protocol is defined, all protocols (except Bang&Olufsen) are active.
 * This must be done before the #include <IRremote.hpp>
 */
//#define DECODE_DENON        // Includes Sharp
//#define DECODE_JVC
//#define DECODE_KASEIKYO
//#define DECODE_PANASONIC    // alias for DECODE_KASEIKYO
//#define DECODE_LG
//#define DECODE_NEC          // Includes Apple and Onkyo. To enable all protocols , just comment/disable this line.
//#define DECODE_SAMSUNG
//#define DECODE_SONY
//#define DECODE_RC5
//#define DECODE_RC6

//#define DECODE_BOSEWAVE
//#define DECODE_LEGO_PF
//#define DECODE_MAGIQUEST
//#define DECODE_WHYNTER
//#define DECODE_FAST

//#define DECODE_DISTANCE_WIDTH // Universal decoder for pulse distance width protocols
//#define DECODE_HASH         // special decoder for all protocols

//#define DECODE_BEO          // This protocol must always be enabled manually, i.e. it is NOT enabled if no protocol is defined. It prevents decoding of SONY!

#define DEBUG               // Activate this for lots of lovely debug output from the decoders.

//#define RAW_BUFFER_LENGTH  750 // For air condition remotes it requires 750. Default is 200.

/*
 * This include defines the actual pin number for pins like IR_RECEIVE_PIN, IR_SEND_PIN for many different boards and architectures
 */
#include "PinDefinitionsAndMore.h"
#include <IRremote.hpp> // include the library

void setup() {
    Serial.begin(115200);
    while (!Serial)
        ; // Wait for Serial to become available. Is optimized away for some cores.

    // Just to know which program is running on my Arduino
    //Serial.println(F("START " __FILE__ " from " __DATE__ "\r\nUsing library version " VERSION_IRREMOTE));

    // Start the receiver and if not 3. parameter specified, take LED_BUILTIN pin from the internal boards definition as default feedback LED
    IrReceiver.begin(IR_RECEIVE_PIN, ENABLE_LED_FEEDBACK);

    //Serial.print(F("Ready to receive IR signals of protocols: "));
    //printActiveIRProtocols(&Serial);
    //Serial.println(F("at pin " STR(IR_RECEIVE_PIN)));
}

void loop() {
    /*
     * Check if received data is available and if yes, try to decode it.
     * Decoded result is in the IrReceiver.decodedIRData structure.
     *
     * E.g. command is in IrReceiver.decodedIRData.command
     * address is in command is in IrReceiver.decodedIRData.address
     * and up to 32 bit raw data in IrReceiver.decodedIRData.decodedRawData
     */
    if (IrReceiver.decode()) {

        /*
         * Print a summary of received data
         */
        if (IrReceiver.decodedIRData.protocol == UNKNOWN) {
            Serial.println(F("Received noise or an unknown (or not yet enabled) protocol"));
            // We have an unknown protocol here, print extended info
            IrReceiver.printIRResultRawFormatted(&Serial, true);
            IrReceiver.resume(); // Do it here, to preserve raw data for printing with printIRResultRawFormatted()
        } else {
            IrReceiver.resume(); // Early enable receiving of the next IR frame
            //IrReceiver.printIRResultRawFormatted(&Serial, true);
            //IrReceiver.printIRResultShort(&Serial);
            //Serial.println();
            //IrReceiver.printIRSendUsage(&Serial);
            //Serial.println();
            //IrReceiver.printIRResultAsCVariables(&Serial);
            //Serial.println();
            IrReceiver.printIRResultMinimal(&Serial);
            Serial.println();
        }
        Serial.println();

        /*
         * Finally, check the received data and perform actions according to the received command
         */
        if (IrReceiver.decodedIRData.command == 0x10) {
            // do something
        } else if (IrReceiver.decodedIRData.command == 0x11) {
            // do something else
        }
    }
}

---------------------------------------------- End of Arduino code ----------------------------------------
********************************************************************************************************** */



//The following is useful info / old rust code used in the development of this project

/* 
fn interact<T: SerialPort>(port: &mut T) -> io::Result<()> {
    port.reconfigure(&|settings| {
    settings.set_baud_rate(serial::Baud115200)?;
    settings.set_char_size(serial::Bits8);
    settings.set_parity(serial::ParityNone);
    settings.set_stop_bits(serial::Stop1);
    settings.set_flow_control(serial::FlowNone);
    Ok(())
    })?;
    
    port.set_timeout(Duration::from_millis(1000))?;
    
    let reader = BufReader::new(port);
    for line in reader.lines() {
        if line.is_ok() {
            println!("{:?}",  line.unwrap_or("Reading failed".into()));
        }
    }
    Ok(())
    }

*/



/*

Minx XiIR Remote Control Codes IR Remote Control Codes
From https://supportarchive.cambridgeaudio.com/hc/en-us/article_attachments/360000080557

    System Code = 27 (RC-5 Code)
Key                     Command code dec.
Memory                  88
Standby Toggle          2
Power on                102
Power off               103
Brightness              51
Reply                   9
Random                  20
Repeat                  21
Info                    28
Mute On                 126
Mute Off                127
Home                    12
Up                      13
Return                  22
Skip Back               23
Left                    16
Select                  17
Right                   18
Skip Forward            25
Stop/Cance              l27
Down                    19
Play/Pause              24
MP3                     32
A1                      33
A2                      34
Bluetooth               31
D1                      35
D2                      36


System Code = 17 (RC-5 Code)
Key                     Command code dec
Internet Radio          111
Services                114
Media                   112
Podcasts                115
Tone/Balance            123
Mute Toggle             67
Volume Up               64
Volume Down             65

---------------- End Minx data -----------------

*/

/*
    match z {
        Ok(_) => {
            //val = z.clone().unwrap(),
            let rc5_1: u32 = z.unwrap();               //represents a RCx7 command 
            let rc5_cmd: u32 = 0b0000000000111111;     //mask for lowest 6 bits to give command   0x3F  0b0000000000111111
            let rc5_addr: u32 = 0b0000011111000000;    //mask for address bits 7-11               0x7C0 0b0000011111000000
            let rc5_tog: u32 = 0b0000100000000000;     //mask for toggle bit 12                   0x800 0b0000100000000000
            let mut cmd: u32 = rc5_1 & rc5_cmd;           //this should give the lowest 6 bits
            let addr: u32 = (rc5_1 & rc5_addr)>>6;    //this should give the address but shifted 6 bits to high 
            let tog: u32 = (rc5_1 & rc5_tog)>>11;     //

            //in the protocol there seems to be a rc5x where an additional +0x40 is added to the command (this seems to align with the Minx sys code 17)
            //since cannot get to raw data from bit stream adding this a work round
            //info from https://github.com/Arduino-IRremote/Arduino-IRremote/blob/master/src/ir_RC5_RC6.hpp

            if addr == 0x11 { cmd += 0x40 }; //address 17 in Dec

            println!("\nData to break into RC5 code is: {:#04X}", rc5_1);
            println!("Dec Address is: {}, Command is: {}, Toggle is: {}", addr, cmd, tog);
            println!("Hex Address is: {:#02X}, Command is: {:#02X}, Toggle is: {}", addr, cmd, tog);

            if addr == 0x11 && rc5_sc17.contains_key(&cmd) {
                println!(" Remote Control command is: {:?}", rc5_sc17.get(&cmd).unwrap());
            }
            else if addr == 0x1B && rc5_sc27.contains_key(&cmd) {
                println!(" Remote Control command is: {:?}", rc5_sc27.get(&cmd).unwrap());
            }
            else { println!("Command code not recognised....") };
        }
        Err(_) => println!("Unable to convert to hex"), 
    }
    */


    /*
    let mut s=String::new();
    print!("Please enter Hex String (4 bytes max): ");
    let _=stdout().flush();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }

    let without_prefix = s.trim_start_matches("0x");
    let z = u32::from_str_radix(without_prefix, 16);
    let _ = decode_rc5_info(z, rc5_sc17, rc5_sc27);

    */

    /*


            let p17 = "P=17".to_string();
            let a1 = "A=".to_string();
            let c1 = "C=".to_string();
            let raw = "Raw=".to_string();
            let fail = "XYZ".to_string();
    if let Some(result) = s1.find(&p17) {
                println!("Found P17 at index: {result}"); 
                //now try to grab chars after 
                println!("Segment we want is: {:?}", s1.get(result..));
            }
            if let Some(result) = s1.find(&a1) {
                println!("Found A= at index: {result}"); 
            }

            if let Some(result) = s1.find(&c1) {
                println!("Found C= at index: {result}"); 
            }
            if let Some(result) = s1.find(&raw) {
                println!("Found Raw= at index: {result}"); 
            }
            if let Some(result) = s1.find(&fail) {
                println!("Found XYZ at index: {result}"); 
            }
     */



            //println!("s1 is: {}", s1);
            //typical line is "P=17 A=0x1B C=0x14 Raw=0x16D4 R"
            /* 
            let mut addr = 0x00;
            let mut cmd = 0x00;
            let mut raw = 0x00;

            if &s1 != "" && (&s1[0..4] == "P=17".to_string()) {
                println!("Data to work on is: {}", &s1);        ////typical line is "P=17 A=0x1B C=0x14 Raw=0x16D4 R"
                println!("s1 5 to 7 is: {}", &s1[5..7]);        //A=
                println!("s1 7 to 11 is: {}", &s1[7..11]);      //A data 0x1B or 0x11
                println!("s1 12 to 14 is: {}", &s1[12..14]);    //C= 
                println!("s1 14 to 18 is: {}", &s1[14..18]);    //C data 0X1 or 0X10 (note need to cope with 3 or 4 digits) 
                println!("s1 19 to 23 is: {}", &s1[19..23]);    //Raw=
                println!("s1 23 to 29 is: {}", &s1[23..29]);    //Raw data as with c may contan 3 or 4 digits
                if &s1 != "" && (&s1[5..7] == "A=".to_string()) {
                    let s = &s1[7..11];
                    let without_prefix = s.trim_start_matches("0x");
                    let z = u32::from_str_radix(without_prefix, 16);
                    addr = z.unwrap();
                }
                if &s1 != "" && (&s1[12..14] == "C=".to_string()) {
                    let s = &s1[14..18];
                    let without_prefix = s.trim_start_matches("0x");
                    let z = u32::from_str_radix(without_prefix, 16);
                    cmd = z.unwrap();
                }
                if &s1 != "" && (&s1[19..23] == "Raw=".to_string()) {
                    let s = &s1[23..29];
                    let without_prefix = s.trim_start_matches("0x");
                    let z = u32::from_str_radix(without_prefix, 16);
                    raw = z.unwrap();
                }
                println!("Addr : {:#04X}, cmd: {:#04X}, raw: {:#04X}", addr, cmd, raw);
                decode_rc5_addr_cmd(addr, cmd, rc5_sc17.clone(), rc5_sc27.clone());
            }

            //lc += 1;
            */
                //sub split each token on the '='
                //let parts: &Vec<_> = &tokens[1].split("=").map(str::to_string).collect();
                //println!("Address Part 1 is {}, part2 is: {}", parts[0], parts[1]); 
                //let without_prefix = parts[1].trim_start_matches("0x");
                //let z = u32::from_str_radix(without_prefix, 16);
                //addr = z.unwrap();
                /*
                                let parts: &Vec<_> = &tokens[2].split("=").map(str::to_string).collect();
                //println!("Command Part 1 is {}, part2 is: {}", parts[0], parts[1]); 
                let without_prefix = parts[1].trim_start_matches("0x");
                let z = u32::from_str_radix(without_prefix, 16);
                cmd = z.unwrap();
                let parts: &Vec<_> = &tokens[3].split("=").map(str::to_string).collect();
                //println!("Raw Part 1 is {}, part2 is: {}", parts[0], parts[1]); 
                let without_prefix = parts[1].trim_start_matches("0x");
                let z = u32::from_str_radix(without_prefix, 16);
                raw = z.unwrap();
                 */
/*
fn decode_rc5_addr_cmd (addr: u32, cmd: u32, rc5_sc17: HashMap<u32, &str>, rc5_sc27: HashMap<u32, &str>) {
    //need to decide what to return if anything
    //for moment this can be decode and print
    //only issue is we don't know toggle status so assume

    //in the protocol there seems to be a rc5x where an additional +0x40 is added to the command (this seems to align with the Minx sys code 17)
    //since cannot get to raw data from bit stream adding this a work round
    //info from https://github.com/Arduino-IRremote/Arduino-IRremote/blob/master/src/ir_RC5_RC6.hpp

    if addr == 0x11 && rc5_sc17.contains_key(&cmd) {
        println!(" Remote Control command is: {:?}", rc5_sc17.get(&cmd).unwrap());
    }
    else if addr == 0x1B && rc5_sc27.contains_key(&cmd) {
        println!(" Remote Control command is: {:?}", rc5_sc27.get(&cmd).unwrap());
    }
    else { println!("Command code not recognised....") };
}

*/

/*
fn decode_rc5_info (rc5_rcd_data: Result<u32, ParseIntError>, rc5_sc17: HashMap<u32, &str>, rc5_sc27: HashMap<u32, &str>) -> Result<String, String> {
    //Decodes raw hex input from keyboard
    match rc5_rcd_data {
        Ok(_) => {
            //val = z.clone().unwrap(),
            let rc5_1: u32 = rc5_rcd_data.unwrap();               //represents a RCx7 command 
            let rc5_cmd: u32 = 0b0000000000111111;     //mask for lowest 6 bits to give command   0x3F  0b0000000000111111
            let rc5_addr: u32 = 0b0000011111000000;    //mask for address bits 7-11               0x7C0 0b0000011111000000
            let rc5_tog: u32 = 0b0000100000000000;     //mask for toggle bit 12                   0x800 0b0000100000000000
            let mut cmd: u32 = rc5_1 & rc5_cmd;           //this should give the lowest 6 bits
            let addr: u32 = (rc5_1 & rc5_addr)>>6;    //this should give the address but shifted 6 bits to high 
            let tog: u32 = (rc5_1 & rc5_tog)>>11;     //

            //in the protocol there seems to be a rc5x where an additional +0x40 is added to the command (this seems to align with the Minx sys code 17)
            //since cannot get to raw data from bit stream adding this a work round
            //info from https://github.com/Arduino-IRremote/Arduino-IRremote/blob/master/src/ir_RC5_RC6.hpp

            if addr == 0x11 { cmd += 0x40 }; //address 17 in Dec

            println!("\nData to break into RC5 code is: {:#04X}", rc5_1);
            println!("Dec Address is: {}, Command is: {}, Toggle is: {}", addr, cmd, tog);
            println!("Hex Address is: {:#02X}, Command is: {:#02X}, Toggle is: {}", addr, cmd, tog);

            if addr == 0x11 && rc5_sc17.contains_key(&cmd) {
                println!(" Remote Control command is: {:?}", rc5_sc17.get(&cmd).unwrap());
            }
            else if addr == 0x1B && rc5_sc27.contains_key(&cmd) {
                println!(" Remote Control command is: {:?}", rc5_sc27.get(&cmd).unwrap());
            }
            else { println!("Command code not recognised....") };

            Ok("Successful".to_string())
        }
        //Err(_) => println!("Unable to convert to hex"), 
        Err(_) => return Err("Unable to convert to hex".to_string())
    }

}

*/