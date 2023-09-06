#!/bin/bash

# Check if the user provided the number of folders to create
if [ $# -ne 1 ]; then
    echo "Usage: $0 <number_of_folders>"
    exit 1
fi

# Get the number of folders to create from command line argument
num_folders="$1"

# Function to generate a random number between a specified range
random_number() {
    min="$1"
    max="$2"
    echo $((RANDOM % (max - min + 1) + min))
}

# Loop to create the specified number of folders
for ((i = 1; i <= num_folders; i++)); do
    # Generate a random folder name
    folder_name="folder_$i"
    mkdir "$folder_name"

    # Generate a random number of files and subdirectories
    num_files=$(random_number 1 10) # Adjust the range as needed
    num_subdirs=$(random_number 0 5) # Adjust the range as needed

    # Create random files
    for ((j = 1; j <= num_files; j++)); do
        touch "$folder_name/file_$j.txt"
    done

    # Create random subdirectories
    for ((k = 1; k <= num_subdirs; k++)); do
        mkdir "$folder_name/subdir_$k"
    done

    echo "Created $folder_name with $num_files files and $num_subdirs subdirectories."
done

echo "Random folder creation completed."
