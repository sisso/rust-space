using System.Collections;
using System.Collections.Generic;
using UnityEngine;

using utils;
using game.domain;

namespace game.ui {


    public class UIController : MonoBehaviour
    {
        public Domain domain;

        public UIPrefabs prefabs;

        private bool hasSectorId;
        private ulong currentSectorId;

        public GameObject buttonsPanel;
        public GameObject sectorMapPanel;

        void Start()
        {
            resolveInitialSectorId();
        }

        public void Refresh()
        {
            if (hasSectorId == false) 
            {
                resolveInitialSectorId();
                if (hasSectorId == false) 
                {
                    Debug.LogWarning("UI has no sector");
                    return;
                }
            }

            sectorMapPanel.transform.CleanUp();

            foreach(var obj in domain.ListObjectsAtSector(currentSectorId)) 
            {

            }
        }

        void resolveInitialSectorId() 
        {
            var sectors = domain.ListSectors();
            if (sectors.Count > 0)
            {
                currentSectorId = sectors[0].id.value;
                hasSectorId = true;
            }
        }

    }
}