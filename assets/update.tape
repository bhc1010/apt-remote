# Output GIF
Output update-demo.gif

# Terminal appearance
Set FontSize 30
Set Width 2040
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
Wait /.*$/
Sleep 2.5s
Type "exit"
Enter
Wait

# --- Generate package image from remote ---
Type "apt-remote set update-demo --target pi@192.168.137.2 --update"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait
Sleep 1s

# --- Download packages on online machine ---
Type "apt-remote get update-demo"
Enter
Wait
Sleep 1s

# --- Install packages on remote machine ---
Type "apt-remote update update-demo --target pi@192.168.137.2"
Enter
Wait /.*:/
Type@10ms "raspberrypi"
Enter 
Wait /.*:/
Type@10ms "raspberrypi"
Enter
Sleep 38s
Wait 

# --- SSH into remote system after installation ---
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
