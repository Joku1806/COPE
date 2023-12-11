import os
import argparse

def print_file_contents(directory):
    for root, _, files in os.walk(directory):
        for file in files:
            file_path = os.path.join(root, file)
            with open(file_path, 'r') as f:
                print(f"File: {file_path}")
                print(f.read())
                print("\n" + "-" * 20 + "\n")

def main():
    parser = argparse.ArgumentParser(description="Print the contents of all files in a directory.")
    parser.add_argument("directory", help="Path to directory")
    args = parser.parse_args()

    directory = args.directory
    if not os.path.exists(directory):
        print(f"Error: Directory '{directory}' does not exist.")
        return

    print_file_contents(directory)

if __name__ == "__main__":
    main()

