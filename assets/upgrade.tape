# Output GIF
Output upgrade-demo.mp4

# Terminal appearance
Set FontSize 32
Set Width 1920
Set Height 1080
Set Theme "Catppuccin Frappe"
Set TypingSpeed 75ms
Set PlaybackSpeed 0.3

# --- Show remote system before installation ---
Type "ssh pi@192.168.137.2"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait /.*$/
Sleep 2.5s
Type "sudo apt-get upgrade"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait /.*\]/
Sleep 1s
Type "n"
Enter
Wait /.*$/
Sleep 3s

Type "exit"
Enter
Wait

# --- Generate package image from remote ---
Type "apt-remote set upgrade-demo --target pi@192.168.137.2 --upgrade"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait
Sleep 1s

# --- Download packages on online machine ---
Type "apt-remote get upgrade-demo"
Enter
Wait
Sleep 2s

# --- Install packages on remote machine ---
Type "apt-remote install upgrade-demo --target pi@192.168.137.2"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait /.*:/
Type@24ms "raspberrypi"
Enter
Sleep 3s
Wait

# --- SSH into remote system after installation ---
Type "ssh pi@192.168.137.2"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait /.*$/
Sleep 200ms
Type "sudo apt-get upgrade"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter
Wait /.*$/
Sleep 2s
