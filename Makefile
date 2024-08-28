MDFILES = $(wildcard */*.md)
# MDFILES := $(filter-out README.md, $(MDFILES))
DIRS_ALL = $(wildcard */)
DIRS = $(filter-out fonts/ template/, $(DIRS_ALL))
PDFFILES = $(MDFILES:.md=.pdf)
PANDOCOPTS = --pdf-engine=lualatex

all:
	$(foreach var,$(DIRS), echo "In $(var)" && make -C $(var) all;)

clean:
	$(foreach var,$(DIRS), make -C $(var) clean;)
