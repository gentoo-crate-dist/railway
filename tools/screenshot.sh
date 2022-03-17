#!/bin/sh
# DO NOT INTERACT WITH THE KEYBOARD WHILE RUNNING, IT MIGHT BREAK THE KEYBOARD FOR APPLICATIONS.

width=1600
width_small=450
height=900

id=$(xdotool search --name DieBahn | tail -n 1)

# Resize Window
xdotool windowsize $id $width $height
xdotool windowmove $id 0 0 

# Time to focus window

# Input Source and Destination
xdotool windowfocus $id 
xdotool type --window $id "Berlin Hauptbahnhof"
xdotool key --window $id Tab
xdotool type --window $id "Münchsmünster"
xdotool key --window $id Tab

# Search
sleep 3
xdotool key --window $id Enter
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Tab
xdotool key --window $id Enter

# Wait for query to finish
sleep 3
xdotool key --window $id Tab
xdotool key --window $id Enter

scrot -s -F overview.png

# Resize Window
xdotool windowsize $id $width_small $height

# Screenshot Journey page
scrot -s -F journey.png

# Return to Journeys
xdotool windowfocus $id 
xdotool key --window $id Tab
xdotool key --window $id Shift+Tab
xdotool key --window $id Shift+Tab
xdotool key --window $id Enter

# Screenshot Journeys page
scrot -s -F journeys.png

# Return to Search
xdotool windowfocus $id 
xdotool key --window $id Tab
xdotool key --window $id Shift+Tab
xdotool key --window $id Shift+Tab
xdotool key --window $id Shift+Tab
xdotool key --window $id Enter

# Screenshot Search page
scrot -s -F search.png

convert +append search.png journeys.png journey.png mobile.png
