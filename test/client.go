package main

import (
	"encoding/json"
	"fmt"
	"net"
)

type ChunkRequest struct {
	FilePath    string `json:"filepath"`
	ChunkNum    int    `json:"chunkNumber"`
	ExpectedSHA string `json:"sha"`
}

func main() {
	conn, err := net.Dial("tcp", "localhost:8080")
	if err != nil {
		fmt.Println("Connection error:", err)
		return
	}
	defer conn.Close()

	request := ChunkRequest{
		FilePath:    "testfile.bin",
		ChunkNum:    1,
		ExpectedSHA: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", // Example SHA256
	}

	requestJSON, _ := json.Marshal(request)
	conn.Write(requestJSON)

	// Read response
	buf := make([]byte, 1024*1024)
	n, err := conn.Read(buf)
	if err != nil {
		fmt.Println("Error receiving:", err)
		return
	}

	fmt.Println("Received chunk:", n, "bytes")
}
