package integration

import (
  "testing"
  "os/exec"
  "os"
  "bytes"
  "time"
)

var cmds = make([]*exec.Cmd, 0)

func TestStartupSerialMessage(t *testing.T) {
  cmd := exec.Command(os.Getenv("ROOT")+"/bin/run")
  cmd.Dir = os.Getenv("ROOT")
  var buf bytes.Buffer
  cmd.Stdout = &buf
  err := cmd.Start()
  if err != nil {
    t.Error(err)
  }
  cmds = append(cmds, cmd)
  <-time.After(500*time.Millisecond)
  msg := string(buf.Bytes())
  needle := "Switched to serial console."
  if len(msg) < len(needle) {
    t.Error("needle is",len(needle),"long, but msg has only",len(msg))
    return
  }
  match := msg[0:len(needle)]
  if needle != match {
    t.Error("expected match:",needle,match)
  }
}


func TestMain(m *testing.M) {
  v := m.Run()
  for k := range cmds {
    cmds[k].Process.Kill()
  }
  os.Exit(v)
}
