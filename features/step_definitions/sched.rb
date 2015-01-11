require 'fileutils'

Before do
  @supplement = []
  @supplement_main ||= ""
end

Given(/^the following C code in the kernel:$/) do |string|
  @supplement << string
  File.write('test_mock_supplement.c~', @supplement.join("\n\n")+"\n\nvoid test_mock_main() {#{@supplement_main}}")
  Subprocess.check_call(["touch", "test_mock_supplement.c~"]) # to trigger make
end

After do
  FileUtils.rm_f('test_mock_supplement.c~')
end

Given(/^I configure the kernel to start a thread with entrypoint "(.*?)"$/) do |func|
  @supplement_main += "sched_add(#{func}, \"Cucumber thread #{func}\");"
  File.write('test_mock_supplement.c~', @supplement.join("\n\n")+"\n\nvoid test_mock_main() {#{@supplement_main}}")
  Subprocess.check_call(["touch", "test_mock_supplement.c~"]) # to trigger make
end

Then(/^I should see "(.*?)" alternating with "(.*?)"$/) do |needle_a, needle_b|
  got_a_b = false
  got_b_a = false
  got_a = false
  got_b = false
  @out = ""
  catch :bye do
    begin
      Timeout.timeout(2) do
        loop do
          l = @process.stdout.gets
          @out << l
          if l.include?(needle_a)
            got_a = true

            if got_b
              got_b_a = true
            end
          elsif l.include?(needle_b)
            got_b = true
            if got_a
              got_a_b = true
            end
          end

          if got_a_b && got_b_a
            # we're good
            throw :bye
          end
        end
      end
    rescue Timeout::Error
      assert false, "expected to find \"#{needle_a}\" alternating with \"#{needle_b}\" within \"#{@out}\""
    end
  end
end

