package paper

import (
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

func TestReadTrailer(t *testing.T) {
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
