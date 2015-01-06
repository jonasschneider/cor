Given(/^I have a disk image with a sector full of "(.*?)"$/) do |marker|
  File.write("cucumberdisk.bin", marker*1000)
end

Given(/^I attach this image as a virtio block device$/) do
  ENV["QEMUOPT"] = "-drive file=cucumberdisk.bin,if=virtio"
end
