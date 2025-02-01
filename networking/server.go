package main

/*
#cgo LDFLAGS: -L. -lfilechunk
#include <stdlib.h>

extern unsigned char* supplier(const char* chunkData, size_t chunk_size);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"unsafe"
)

// Request structure (JSON)
type ChunkRequest struct {
	FilePath    string `json:"filepath"`
	ChunkNum    int    `json:"chunkNumber"`
	ExpectedSHA string `json:"sha"`
}

// Handle client requests
func handleConnection(conn net.Conn) {
	defer conn.Close()

	// Read JSON request
	buf := make([]byte, 1024)
	n, err := conn.Read(buf)
	if err != nil {
		fmt.Println("Error reading:", err)
		return
	}

	var req ChunkRequest
	err = json.Unmarshal(buf[:n], &req)
	if err != nil {
		fmt.Println("Invalid JSON:", err)
		conn.Write([]byte("Invalid request\n"))
		return
	}

	// Convert request to JSON string
	requestJSON, _ := json.Marshal(req)
	cstr := C.CString(string(requestJSON))
	defer C.free(unsafe.Pointer(cstr))

	// Call Rust function (request chunk)
	chunkPtr := C.supplier(cstr, 1024*1024) // 1MB chunk size
	if chunkPtr == nil {
		conn.Write([]byte("Error: Invalid chunk request\n"))
		return
	}

	// Read chunk bytes into Go slice
	chunkBytes := C.GoBytes(unsafe.Pointer(chunkPtr), C.int(1024*1024))
	conn.Write(chunkBytes) // Send chunk over TCP

	fmt.Println("Sent chunk", req.ChunkNum, "of", req.FilePath)
}

func main() {
	listener, err := net.Listen("tcp", "localhost:8080")
	if err != nil {
		fmt.Println("Failed to start server:", err)
		os.Exit(1)
	}
	defer listener.Close()

	fmt.Println("Server started on localhost:8080")
	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Connection error:", err)
			continue
		}
		go handleConnection(conn) // Handle each client concurrently
	}
}
