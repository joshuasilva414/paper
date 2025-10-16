package paper

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"testing"
)

func TestTrivialTrailer(t *testing.T) {
	testData, err := ReadTestData()
	if err != nil {
		t.Errorf("Failed to read test data: %v", err)
	}

	for _, testFile := range testData.Data {
		file, err := os.Open(filepath.Join("testdata", testFile.Path))
		if err != nil {
			t.Errorf("Failed to open test file: %v", err)
		}
		defer file.Close()

		offset, err := FindStartXref(file)
		if err != nil {
			t.Errorf("Failed to find startxref in file %s: %v", testFile.Path, err)
		}

		t.Logf("Trailer offset: %d", offset)
	}
}

func TestFindStartXref(t *testing.T) {
	testData, err := ReadTestData()
	if err != nil {
		t.Errorf("Failed to read test data: %v", err)
	}

	for _, testFile := range testData.Data {
		file, err := os.Open(filepath.Join("testdata", testFile.Path))
		if err != nil {
			t.Errorf("Failed to open test file: %v", err)
		}
		defer file.Close()

		offset, err := FindStartXref(file)
		if err != nil {
			t.Errorf("Failed to find startxref in file %s: %v", testFile.Path, err)
		}

		PrintNBytesAtOffset(t, file, offset, 32)

		t.Logf("Startxref: %d", offset)
	}
}

func TestXrefType(t *testing.T) {
	testData, err := ReadTestData()
	if err != nil {
		t.Errorf("Failed to read test data: %v", err)
	}

	for _, testFile := range testData.Data {
		file, err := os.Open(filepath.Join("testdata", testFile.Path))
		if err != nil {
			t.Errorf("Failed to open test file: %v", err)
		}
		defer file.Close()

		offset, err := FindStartXref(file)
		if err != nil {
			t.Errorf("Failed to find startxref in file %s: %v", testFile.Path, err)
		}

		_, seekErr := file.Seek(offset, io.SeekStart)
		if seekErr != nil {
			t.Errorf("Failed to seek to startxref in file %s: %v", testFile.Path, seekErr)
		}

		buffer := make([]byte, 32)
		_, readErr := file.Read(buffer)
		if readErr != nil {
			t.Errorf("Failed to read from file %s: %v", testFile.Path, readErr)
		}

		var xRefType string
		if bytes.HasPrefix(buffer, []byte("xref")) {
			xRefType = "table"
		} else {
			xRefType = "stream"
		}

		t.Logf("xref type: %s", xRefType)
		PrintNBytesAtOffset(t, file, offset, 32)
	}
}
