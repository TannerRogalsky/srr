#[test]
fn parse_pyrescenes() {
    let files = vec![
        "pyrescene_test_files/best_little/added_empty_file.srr",
        "pyrescene_test_files/bug_detected_as_being_different/The.First.Great.Train.Robbery.1978.iNTERNAL.DVDRip.XviD-EXViDiNT_nzbsauto.srr",
        "pyrescene_test_files/bug_detected_as_being_different/The.First.Great.Train.Robbery.1978.iNTERNAL.DVDRip.XviD-EXViDiNT_yopom.srr",
        "pyrescene_test_files/bug_detected_as_being_different2/Top.Gear.S01E09.TVRiP.DiVX.iNTERNAL-GRiM_new.srr",
        "pyrescene_test_files/bug_detected_as_being_different2/Top.Gear.S01E09.TVRiP.DiVX.iNTERNAL-GRiM_orig.srr",
        "pyrescene_test_files/bug_detected_as_being_different3/Akte.2012.08.01.German.Doku.WS.dTV.XViD-FiXTv_f4n4t.srr",
        "pyrescene_test_files/bug_detected_as_being_different3/Akte.2012.08.01.German.Doku.WS.dTV.XViD-FiXTv_nzbsauto.srr",
        "pyrescene_test_files/bug_detected_as_being_different3/The.Closer.S04E10.Zeitbomben.German.WS.DVDRip.XviD-EXPiRED_f4n4t.srr",
        "pyrescene_test_files/bug_detected_as_being_different3/The.Closer.S04E10.Zeitbomben.German.WS.DVDRip.XviD-EXPiRED_nzbsauto.srr",
        "pyrescene_test_files/cleanup_script/007.A.View.To.A.Kill.1985.UE.iNTERNAL.DVDRip.XviD-iNCiTE.fine_2cd.srr",
        "pyrescene_test_files/cleanup_script/007.Quantum.Of.Solace.DVDRip.XViD-PUKKA.cleanup_script.srr",
        "pyrescene_test_files/cleanup_script/fixed/007.A.View.To.A.Kill.1985.UE.iNTERNAL.DVDRip.XviD-iNCiTE.fine_2cd.srr",
        "pyrescene_test_files/cleanup_script/fixed/007.Quantum.Of.Solace.DVDRip.XViD-PUKKA.cleanup_script.srr",
        "pyrescene_test_files/hash_capitals/Parlamentet.S06E02.SWEDiSH-SQC_alllower.srr",
        "pyrescene_test_files/hash_capitals/Parlamentet.S06E02.SWEDiSH-SQC_capitals.srr",
        "pyrescene_test_files/incomplete_srr/Shark.Week.2012.Shark.Fight.HDTV.x264-KILLERS.srr",
        "pyrescene_test_files/no_files_stored/Burial.Ground.The.Nights.of.Terror.1981.DVDRip.XviD-spawny.srr",
        "pyrescene_test_files/no_files_stored/Hofmanns.Potion.2002.DVDRip.XviD-belos.srr",
        "pyrescene_test_files/no_files_stored/Zombi.Holocaust.1980.DVDRip.XviD-spawny.srr",
        "pyrescene_test_files/other/007.A.View.To.A.Kill.1985.UE.iNTERNAL.DVDRip.XviD-iNCiTE_fine_2cd.srr",
        "pyrescene_test_files/other/007.Die.Another.Day.2002.iNTERNAL.DVDRip.XviD-iNCiTE.2CD_paths_stored.srr",
        "pyrescene_test_files/other/007.Quantum.Of.Solace.DVDRip.XViD-PUKKA_cleanup_script.srr",
        "pyrescene_test_files/other/24.S08E03.720p.BluRay.X264-WASABi_srrpathhack.srr",
        "pyrescene_test_files/other/Antz.1998.iNTERNAL.DVDRip.XviD-SLeTDiVX.srr",
        "pyrescene_test_files/other/Blood.2009.1080p.BluRay.x264-Japhson.srr",
        "pyrescene_test_files/other/David.Letterman.2011.12.09.David.Duchovny.HDTV.XviD-2HD.srr",
        "pyrescene_test_files/other/Dexter.S05E02.iNTERNAL.720p.HDTV.x264-ORENJI_vlcbugreport.srr",
        "pyrescene_test_files/other/Dragons.Den.S06E06.WS.PDTV.XviD-SPAREL_no_app_name.srr",
        "pyrescene_test_files/other/Enterprise.4x01.Storm_Front_Part1.DVDRip_XviD-FoV.srr",
        "pyrescene_test_files/other/Enterprise.4x02.Storm_Front_Part2.DVDRip_XviD-FoV.srr",
        "pyrescene_test_files/other/Farscape.S01E01.AC3.DivX.DVDRip.iNTERNAL-AMC_old_style_rr.srr",
        "pyrescene_test_files/other/Father.Of.The.Bride.1.1991.INTERNAL.DVDRip.XviD-PtSL_sample_norebuild.srr",
        "pyrescene_test_files/other/Game.of.Thrones.S01E02.The.Kingsroad.HDTV.XviD-FQM_vlc.srr",
        "pyrescene_test_files/other/Game.of.Thrones.S01E07.HDTV.XviD-ASAP.srr",
        "pyrescene_test_files/other/House.S06E12.720p.HDTV.x264-IMMERSE_rarfs_problem.srr",
        "pyrescene_test_files/other/Jochem.Myjer.De.Rust.Zelve.2010.DUTCH.DVDRip.XviD-INViTED_sfv_case.srr",
        "pyrescene_test_files/other/The.Shawshank.Redemption.1994.720p.BluRay.x264-SiNNERS_empty_sfv.srr",
        "pyrescene_test_files/other/The.cLoser.S05E56.DVDRip.XviD-TOPAZ - kopie.srr",
        "pyrescene_test_files/other/The.more.cLoser.S05E56.DVDRip.XviD-TOPAZ.srr",
        "pyrescene_test_files/other/Zombi.Holocaust.1980.DVDRip.XviD-spawny_nofiles.srr",
        "pyrescene_test_files/other/californication.s04e01.proper.hdtv.xvid-asap.srr",
        "pyrescene_test_files/other/house.713.hdtv-lol.srr",
        "pyrescene_test_files/store_empty/added_empty_file.srr",
        "pyrescene_test_files/store_empty/store_empty.srr",
        "pyrescene_test_files/store_little/store_little.srr",
        "pyrescene_test_files/store_little/store_little_srrfile_with_path.srr",
        "pyrescene_test_files/store_little/store_little_srrfile_with_path_backslash.srr",
        "pyrescene_test_files/store_rr_solid_auth_unicode_new/store_rr_solid_auth.part1.srr",
        "pyrescene_test_files/store_split_folder_old_srrsfv_windows/store_split_folder.srr",
        "pyrescene_test_files/store_split_folder_old_srrsfv_windows/winrar2.80.srr",
        "pyrescene_test_files/store_utf8_comment/store_utf8_comment.srr",
        "pyrescene_test_files/store_utf8_comment/utf8_filename_added.srr",
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    for file in files {
        // println!("{}", file);
        let input = std::fs::read(root.join(file)).unwrap();
        let (rest, _srr) = srr::Srr::new(&input).expect(file);
        assert!(rest.is_empty());
    }
}
