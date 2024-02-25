TARGET=proof.pdf
TEXFILES=main.tex proof.tex


default: ${TARGET}

all: ${TARGET}


proof.pdf:  ${TEXFILES} 
	pdflatex -interaction=nonstopmode $<


clean:
	-rm -f *.aux *.out *.log *.dvi
	-rm -f *.bbl *.blg
	-rm -f *.fls
	-rm -f *.fdb_latexmk

clean_pdf:
	-rm -f ${TARGET}

