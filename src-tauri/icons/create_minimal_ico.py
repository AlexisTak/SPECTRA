#!/usr/bin/env python3
"""
Create a minimal valid ICO file from a 1x1 PNG.
This is a workaround for Tauri's requirement of an icon.ico file.
"""

import struct

def create_minimal_ico():
    # Create a minimal valid 1x1 PNG
    # Signature
    png_data = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk (13 bytes)
    width, height = 1, 1
    png_data += struct.pack('>I', 13)  # chunk length
    png_data += b'IHDR'
    png_data += struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)  # 8-bit RGB
    import zlib
    png_data += struct.pack('>I', zlib.crc32(png_data[-22:]))  # CRC
    
    # IEND chunk
    png_data += struct.pack('>I', 0)  # chunk length
    png_data += b'IEND'
    png_data += struct.pack('>I', 0xae426082)  # CRC
    
    # Now wrap in ICO format
    # ICO header: 0,0,1,0,1,0 (reserved, type=images, count=1)
    ico_data = struct.pack('<HH', 0, 1)
    
    # Image directory entry for 1x1 icon
    # 16 bytes: width(1), height(1), colors(0), reserved(0), bytes(9+16), offset(6+16)
    ico_data += struct.pack('<BBIIII',
        0,  # width (0 means 256, but we'll use 1 here for minimal)
        0,  # height (0 means 256)
        0,  # colors
        0,  # reserved
        len(png_data) + 16 + 6,  # bytes in res
        22  # offset to image data
    )
    
    ico_data += png_data
    
    with open('icon.ico', 'wb') as f:
        f.write(ico_data)
    
    print(f"Created minimal ico file: {len(ico_data)} bytes")

if __name__ == '__main__':
    create_minimal_ico()
