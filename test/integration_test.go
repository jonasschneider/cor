package integration

import (
  "testing"
  "os/exec"
  "os"
  "bufio"
  "time"
  "io"
)

var cmds = make([]*exec.Cmd, 0)

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

  rcv := make(chan string)
  go func() {
    s, err := lines.ReadBytes('\n')
    if err != nil {
      t.Error(err)
    }
    rcv <- string(s)
  }()

  cmds = append(cmds, cmd)
  select {
  case msg := <- rcv:
    needle := "Switched to serial console."
    if len(msg) < len(needle) {
      t.Error("needle is",len(needle),"long, but msg has only",len(msg))
      return
    }
    match := msg[0:len(needle)]
    if needle != match {
      t.Error("expected match:",needle,match)
    }
  case <-time.After(1000*time.Millisecond):
    t.Error("timed out without reading anything")
  }
}


func TestMain(m *testing.M) {
  v := m.Run()
  for k := range cmds {
    cmds[k].Process.Kill()
  }
  os.Exit(v)
}
