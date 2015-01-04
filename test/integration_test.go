package test

import (
  "testing"
  "os/exec"
  "os"
  "bufio"
  "time"
  "io"
)

func TestStartupSerialMessage(t *testing.T) {
  cmd := exec.Command(os.Getenv("ROOT")+"/bin/run")
  cmd.Dir = os.Getenv("ROOT")

  rd, wr := io.Pipe()
  lines := bufio.NewReader(rd)
  cmd.Stdout = wr

  err := cmd.Start()
  if err != nil {
    t.Error(err)
  }
  defer func() {
    cmd.Process.Kill()
  }()

  rcv := make(chan string)
  go func() {
    s, err := lines.ReadBytes('\n')
    if err != nil {
      t.Error(err)
    }
    rcv <- string(s)
  }()

  select {
  case msg := <- rcv:
    needle := "Switched to serial console."
    if len(msg) < len(needle) {
      t.Error("needle is",len(needle),"long, but msg has only",len(msg))
    }
    match := msg[0:len(needle)]
    if needle != match {
      t.Error("no match:",match,needle)
    }
  case <-time.After(1000*time.Millisecond):
    t.Error("timed out without reading anything")
  }
}
