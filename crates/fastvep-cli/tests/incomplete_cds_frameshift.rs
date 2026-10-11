use std::{fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

#[test]
fn terminal_deletion_in_incomplete_cds_does_not_panic() {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("fastvep-phase-{}-{}", std::process::id(), nonce));
    fs::create_dir(&root).unwrap();
    // GFF phase is the number of bases to skip; the CDS coordinate offset is
    // (3 - phase) % 3. Keep the padded CDS a whole number of codons.
    for (gff_phase, padding) in [(0, 0), (1, 2), (2, 1)] {
        let n = 1002 - padding;
        let dir = root.join(gff_phase.to_string());
        fs::create_dir(&dir).unwrap();
        let fasta = dir.join("ref.fa");
        fs::write(&fasta, format!(">1\n{}GT\n", "A".repeat(n - 2))).unwrap();
        let gff = dir.join("model.gff3");
        fs::write(&gff, format!("##gff-version 3\n1\ttest\tgene\t1\t{n}\t.\t+\t.\tID=gene:GENE1;biotype=protein_coding\n1\ttest\tmRNA\t1\t{n}\t.\t+\t.\tID=transcript:TX1;Parent=gene:GENE1;biotype=protein_coding\n1\ttest\texon\t1\t{n}\t.\t+\t.\tParent=transcript:TX1;exon_id=EX1;rank=1\n1\ttest\tCDS\t1\t{n}\t.\t+\t{gff_phase}\tID=CDS:PROT1;Parent=transcript:TX1\n")).unwrap();
        let vcf = dir.join("input.vcf");
        fs::write(&vcf, format!("##fileformat=VCFv4.2\n##contig=<ID=1,length={n}>\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n1\t{}\tterminal\tGT\tG\t.\tPASS\t.\n", n - 1)).unwrap();
        let output = Command::new(std::env::var_os("FASTVEP_TEST_BINARY").unwrap_or_else(|| env!("CARGO_BIN_EXE_fastvep").into()))
            .args(["annotate", "--input"]).arg(&vcf)
            .arg("--gff3").arg(&gff).arg("--fasta").arg(&fasta)
            .args(["--hgvs", "--output-format", "vcf", "--output", "-"])
            .output().unwrap();
        assert!(output.status.success(), "phase {gff_phase}: {}", String::from_utf8_lossy(&output.stderr));
        let text = String::from_utf8(output.stdout).unwrap();
        let rows: Vec<_> = text.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(rows.len(), 1, "{text}");
        assert!(rows[0].contains("frameshift_variant"), "{text}");
        assert!(rows[0].contains("TX1"), "{text}");
    }
    fs::remove_dir_all(root).unwrap();
}
