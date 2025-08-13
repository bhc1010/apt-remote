# Output GIF
Output install-demo.gif

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
Sleep 1.5s
Type "neofetch"
Enter
Wait /.*$/
Sleep 1s

Type "exit"
Enter
Wait

# --- Generate package image from remote ---
Type "apt-remote set install-demo --target pi@192.168.137.2 --install neofetch"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait
Sleep 1s

# --- Download packages on online machine ---
Type "apt-remote get install-demo"
Enter
Wait
Sleep 2s

# --- Install packages on remote machine ---
Type "apt-remote install install-demo --target pi@192.168.137.2"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait /.*:/
Type@24ms "raspberrypi"
Enter
Wait

# --- SSH into remote system after installation ---
Type "ssh pi@192.168.137.2"
Enter
Wait /.*:/
Type@24ms "raspberrypi"
Enter 
Wait /.*$/
Sleep 2.5s
Type "neofetch"
Enter
Wait /.*$/
Sleep 3s
