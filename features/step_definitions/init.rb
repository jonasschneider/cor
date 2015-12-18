require 'subprocess'
require 'timeout'
require 'minitest'

Given(/^the following code for \/sbin\/init:$/) do |string|
  File.write("userspace/init.c", string)
  Subprocess.check_call(["touch", "userspace/init.c"]) # to trigger make
end

Given(/^I use the following linker script for init:$/) do |string|
  File.write("userspace/init.ld", string)
  Subprocess.check_call(["touch", "userspace/init.ld"]) # to trigger make
end

When(/^I run the machine$/) do
  mk = Subprocess.check_output(%w(uname)).chomp == "Darwin" ? "vagrant ssh -- cd /vagrant && make" : "make"
  Subprocess.check_call(mk.split(" "))
  if @process
    @process.terminate
    @process.wait
  end
  q = "qemu-system-x86_64 -s -nographic -serial stdio -monitor null disk.bin #{ENV["QEMUOPT"]} -drive file=disk.bin,if=virtio"
  @process = Subprocess.popen(q.split(" "), stdin: nil, stdout: Subprocess::PIPE, stderr: Subprocess::PIPE)
end

Then(/^I should see "([^"]*?)"$/) do |needle|
  @out = ""
  catch :bye do
    begin
      Timeout.timeout(2) do
        loop do
          l = @process.stdout.gets
          @out << l
          if @out.include?(needle)
            throw :bye
          end
        end
      end
    rescue Timeout::Error
      @out = @out.force_encoding('ASCII-8BIT')
      assert @out.include?(needle), "expected to find \"#{needle}\" in \"#{@out}\""
    end
  end
end

Before do
  mk = (ENV["MAKE"]||"make").split(" ")
  Subprocess.check_call(mk.concat(["clean"]))
end

After do
  if @process
    @process.terminate
    @process.wait
  end
end
