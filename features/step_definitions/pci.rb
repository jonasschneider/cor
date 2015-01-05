Then(/^I don't see "(.*?)"$/) do |needle|
  @out = ""
  catch :bye do
    begin
      Timeout.timeout(2) do
        loop do
          l = @process.stdout.gets
          @out << l
          if @out.include?(needle)
            assert false, "expected NOT to find \"#{needle}\" in \"#{@out}\""
          end
        end
      end
    rescue Timeout::Error
      # ok
    end
  end
end

Given(/^I attach a virtio network interface to the machine$/) do
  ENV["QEMUOPT"] = "-net nic,model=virtio"
end

After do
  ENV.delete("QEMUOPT")
end
