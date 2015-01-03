require 'subprocess'
require 'timeout'
require 'minitest'

Given(/^the following code for \/sbin\/init:$/) do |string|
  File.write("init.c", string)
end

When(/^I run the machine$/) do
  Subprocess.check_call(["make"])
  @process = Subprocess.popen(["bin/run"], stdout: Subprocess::PIPE)
end

Then(/^I should see "(.*?)"$/) do |needle|
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
      assert @out.include?(needle), "expected to find \"#{needle}\" in \"#{@out}\""
    end
  end
end

After do
  @process.terminate
end
