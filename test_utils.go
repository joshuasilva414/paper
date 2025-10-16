package paper

import (
	"io"
	"os"
	"testing"
)

func PrintNBytesAtOffset(t *testing.T, file *os.File, offset int64, n int) error {
	_, seekErr := file.Seek(offset, io.SeekStart)
	if seekErr != nil {
		t.Errorf("Failed to seek to offset %d: %v", offset, seekErr)
	}

	buffer := make([]byte, n)
	_, readErr := file.Read(buffer)
	if readErr != nil {
		t.Errorf("Failed to read from file %s: %v", file.Name(), readErr)
	}

	t.Logf("Buffer: %s", string(buffer))

	return nil
}
