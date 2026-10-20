// Research only: local synthetic session controls; no authentication or durability.
package main
import (
    "bufio"
    "io"
    "net"
    "os"
)
func serve(conn net.Conn, slots chan struct{}) {
    defer func() { conn.Close(); <-slots }()
    approved := false
    // Fixed command accumulator and bounded reader; writes block only this session.
    input := make([]byte, 0, 1024)
    reader := bufio.NewReaderSize(conn, 256)
    for {
        b, err := reader.ReadByte()
        if err != nil { return }
        if b != '\n' {
            if b < 32 || b > 126 || len(input) >= 1024 { return }
            input = append(input, b); continue
        }
        response := "ERROR\n"
        switch string(input) {
        case "ALLOW": approved = true; response = "OK\n"
        case "REVOKE": approved = false; response = "OK\n"
        case "READ", "POST", "CHAT":
            if !approved { response = "DENIED\n"
            } else if string(input) == "READ" { response = "ROOM synthetic local message\n"
            } else if string(input) == "POST" { response = "POSTED synthetic only\n"
            } else { response = "CHAT synthetic only\n" }
        }
        if _, err = io.WriteString(conn, response); err != nil { return }
        input = input[:0]
    }
}
func main() {
    listener, err := net.Listen("unix", os.Args[1])
    if err != nil { panic(err) }
    defer listener.Close()
    slots := make(chan struct{}, 250)
    for {
        conn, err := listener.Accept()
        if err != nil { panic(err) }
        select {
        case slots <- struct{}{}: go serve(conn, slots)
        default: io.WriteString(conn, "FULL\n"); conn.Close()
        }
    }
}
