package paper

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"
)

func BytesMatchAt(file *os.File, offset int64, expected []byte) (bool, error) {
	buffer := make([]byte, len(expected))

	if _, err := file.ReadAt(buffer, offset); err != nil {
		if err == io.EOF {
			return false, nil
		}
		return false, err
	}

	return bytes.Equal(buffer, expected), nil
}

func FindStartXref(file *os.File) (int64, error) {
	_, seekErr := file.Seek(-100, io.SeekEnd)
	if seekErr != nil {
		return 0, fmt.Errorf("failed to seek: %v", seekErr)
	}

	reader := bufio.NewReaderSize(file, 10)

	for line, _, lineErr := reader.ReadLine(); lineErr == nil; line, _, lineErr = reader.ReadLine() {
		if bytes.HasSuffix(line, []byte("startxref")) {
			numLine, _, err := reader.ReadLine()
			if err != nil {
				return 0, fmt.Errorf("failed to read number line: %v", err)
			}

			num, err := strconv.ParseInt(strings.TrimSpace(string(numLine)), 10, 64)
			if err != nil {
				return 0, fmt.Errorf("failed to convert string to int: %v", err)
			}

			return num, nil
		}
	}
	return 0, fmt.Errorf("startxref not found")
}
